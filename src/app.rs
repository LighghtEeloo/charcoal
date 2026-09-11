pub mod builder;
pub mod cache;
pub mod cli;
pub mod config;

use super::*;

pub struct App {}

impl App {
    pub async fn main() -> anyhow::Result<()> {
        env_logger::init();

        match Cli::new() {
            Commands::Query(args) => App::query(args).await,
            Commands::Edit(args) => App::edit(args).await,
            Commands::Cache { commands } => App::cache(commands).await,
        }
    }

    pub async fn query(mut args: cli::QueryArgs) -> anyhow::Result<()> {
        let app_builder = AppBuilder::new();

        let mut config = app_builder.config()?;
        let cache = app_builder.cache()?;

        config.apply(&mut args);

        let word_query = {
            let word_query = ExactQuery::new(args.query(), config.lang, args.refresh);
            if let Some(word_query) = word_query {
                word_query
            } else {
                println!("Invalid input.");
                return Ok(());
            }
        };

        let (word_entry, audio) = lookup_with_prefetch(
            SingleEntry::query(&word_query, &cache),
            Speech::prepare(&word_query, &cache, config.speak),
            |entry| entry.pprint(&word_query, &config),
        )
        .await?;

        if word_entry.not_found() {
            Suggestion::new(word_query.word()).exec().await?;
            return Ok(());
        }
        if let Some(audio) = audio {
            let result = match audio {
                Ok(Some(audio)) => Speech::speak(audio).await,
                Ok(None) => Ok(()),
                Err(err) => Err(err),
            };
            if let Err(err) = result {
                log::error!("An error occurred in the speech module: {err:#}");
            }
        }

        Ok(())
    }

    pub async fn edit(args: cli::EditArgs) -> anyhow::Result<()> {
        use tokio::process::Command;

        let editor = std::env::var("EDITOR").map_err(|err| {
            println!("Please set $EDITOR to your preferred editor.");
            err
        })?;
        let config_path = {
            let app_builder = AppBuilder::new();
            if args.reset {
                app_builder.config_fresh()?
            } else {
                app_builder.config()?
            }
            .path
        };

        let status = Command::new(editor).args([config_path]).status().await?;
        anyhow::ensure!(status.success(), "Editor exited with {status}");
        Ok(())
    }

    pub async fn cache(cmds: cli::CacheCmds) -> anyhow::Result<()> {
        let cache = AppBuilder::new().cache()?;
        match cmds {
            cli::CacheCmds::Show => {
                println!("{}", cache.show().display());
            }
            cli::CacheCmds::Clean => {
                cache.clean()?;
            }
            cli::CacheCmds::Import { dir } => {
                log::info!("Importing:\n\t<== {}", dir.display());
                cache.import(dir)?;
            }
            cli::CacheCmds::Export { dir } => {
                log::info!("Exporting:\n\t==> {}", dir.display());
                cache.export(dir)?;
            }
        }
        Ok(())
    }
}

/// Poll both operations, render as soon as lookup completes, and cancel prefetch
/// when lookup fails or returns no result. Audio playback remains a separate step.
async fn lookup_with_prefetch<L, P, T>(
    lookup: L, prefetch: P, render: impl FnOnce(&SingleEntry),
) -> anyhow::Result<(SingleEntry, Option<T>)>
where
    L: std::future::Future<Output = anyhow::Result<SingleEntry>>,
    P: std::future::Future<Output = T>,
{
    tokio::pin!(lookup, prefetch);
    let mut ready = None;
    let entry = tokio::select! {
        result = &mut lookup => result?,
        result = &mut prefetch => {
            ready = Some(result);
            lookup.await?
        }
    };
    if entry.not_found() {
        return Ok((entry, None));
    }
    render(&entry);
    let audio = match ready {
        Some(audio) => audio,
        None => prefetch.await,
    };
    Ok((entry, Some(audio)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        future::pending,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    };
    use tokio::sync::oneshot;

    fn entry() -> SingleEntry {
        SingleEntry {
            pronunciation: vec![],
            brief: vec!["A greeting".into()],
            variants: vec![],
            authority: vec![],
            sentence: vec![],
        }
    }

    #[tokio::test]
    async fn prefetch_runs_concurrently_and_text_precedes_audio_completion() {
        let (started, wait_started) = oneshot::channel();
        let (rendered, wait_rendered) = oneshot::channel();
        let lookup = async {
            wait_started.await.unwrap();
            Ok(entry())
        };
        let prefetch = async {
            started.send(()).unwrap();
            wait_rendered.await.unwrap();
            "audio"
        };
        let (_, audio) = tokio::time::timeout(
            Duration::from_secs(2),
            lookup_with_prefetch(lookup, prefetch, |_| {
                rendered.send(()).unwrap();
            }),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(audio, Some("audio"));
    }

    #[tokio::test]
    async fn completed_prefetch_waits_for_lookup_before_rendering() {
        let (ready, wait_ready) = oneshot::channel();
        let lookup = async {
            wait_ready.await.unwrap();
            tokio::task::yield_now().await;
            Ok(entry())
        };
        let prefetch = async {
            ready.send(()).unwrap();
            "audio"
        };
        let rendered = AtomicBool::new(false);
        let (_, audio) = lookup_with_prefetch(lookup, prefetch, |_| {
            rendered.store(true, Ordering::SeqCst);
        })
        .await
        .unwrap();
        assert!(rendered.load(Ordering::SeqCst));
        assert_eq!(audio, Some("audio"));
    }

    struct Cancellation(Arc<AtomicBool>);
    impl Drop for Cancellation {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn failed_and_missing_lookups_cancel_prefetch_without_rendering() {
        for fail in [false, true] {
            let cancelled = Arc::new(AtomicBool::new(false));
            let guard = Cancellation(cancelled.clone());
            let prefetch = async move {
                let _guard = guard;
                pending::<()>().await;
            };
            let lookup = async {
                if fail {
                    anyhow::bail!("Lookup failed");
                }
                let mut entry = entry();
                entry.brief.clear();
                Ok(entry)
            };
            let result = tokio::time::timeout(
                Duration::from_secs(2),
                lookup_with_prefetch(lookup, prefetch, |_| {
                    panic!("Empty results must not render")
                }),
            )
            .await
            .unwrap();
            assert_eq!(result.is_err(), fail);
            if let Ok((_, audio)) = result {
                assert!(audio.is_none());
            }
            assert!(cancelled.load(Ordering::SeqCst));
        }
    }
}
