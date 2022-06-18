use charcoal::{cli, AppBuilder, Cli, Command, Speech, WordEntry};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    let word = args.query;
    if let Some(speak) = args.speak {
        speak.twitch(&mut config.speak)
    }

    let word_speech = Speech::spawn(word.to_owned(), cache.to_owned(), config.speak);
    let word_query = WordEntry::query(&cache, &word).await?;

    if word_query.is_empty() {
        println!("Word not found.");
        return Ok(());
    }

    word_query.display(&word, &config);
    if let Err(err) = word_speech.await {
        log::error!("An error occured in speech module: {:?}.", err)
    }

    Ok(())
}

async fn edit_main(args: cli::EditArgs) -> anyhow::Result<()> {
    use std::process::Command;

    let editor = std::env!("EDITOR");
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
    let mut cache = AppBuilder::new().cache()?;
    cache.clean()
}

/* TODO
 * 4. Authority
 * 6. Better cache consistency with audio
 * 7. Cli
 * 8. en_us / zh_cn
 * 9. concise mode
 */
