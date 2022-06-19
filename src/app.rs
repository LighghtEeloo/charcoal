pub mod builder;
pub mod cache;
pub mod cli;
pub mod config;

use super::*;

pub async fn main() -> anyhow::Result<()> {
    env_logger::init();

    match Cli::new() {
        Command::Query(args) => query_main(args).await,
        Command::Edit(args) => edit_main(args).await,
        Command::Clean => clean_main().await,
    }
}

async fn query_main(args: cli::QueryArgs) -> anyhow::Result<()> {
    let app_builder = AppBuilder::new();

    let mut config = app_builder.config()?;
    let cache = app_builder.cache()?;

    let word = args.query.to_owned();
    config.apply(args);

    let lang = whatlang::detect(word.as_ref())
        .expect("Language detection failed.")
        .lang();
    let word_speech = Speech::query(&word, &lang, &cache, config.speak);
    let word_entry = WordEntry::query(&word, &lang, &cache).await?;

    if word_entry.is_empty() {
        println!("Word not found.");
        return Ok(());
    }

    word_entry.display(&word, &lang, &config);
    if let Err(err) = word_speech.await {
        log::error!("An error occured in speech module: {:?}.", err)
    }

    Ok(())
}

async fn edit_main(args: cli::EditArgs) -> anyhow::Result<()> {
    use std::process::Command;

    let editor = std::env::var("EDITOR").or_else(|err| {
        println!("Please set $EDITOR to your prefered editor.");
        Err(err)
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

    let mut child = Command::new(editor).args([config_path]).spawn()?;
    child.wait()?;
    Ok(())
}

async fn clean_main() -> anyhow::Result<()> {
    let cache = AppBuilder::new().cache()?;
    let res = cache.clean()?;
    Ok(res)
}
