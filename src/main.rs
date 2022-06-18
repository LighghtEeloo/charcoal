use charcoal::{cli, query, AppBuilder, Cli, Command, Speech};

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
    let mut cache = app_builder.cache()?;

    if let Some(speak) = args.speak {
        speak.twitch(&mut config.speak)
    }

    let word = args.query;
    let speech = Speech::new(&config);

    let word_speech = speech.speak(&word);
    let word_query = {
        if let Ok(word_query) = query::FromCache::new(&mut cache).query(&word).await {
            word_query
        } else {
            let word_query = query::FromYoudict::new().query(&word).await?;
            cache.store(&word, word_query.clone())?;
            word_query
        }
    };

    if word_query.is_empty() {
        println!("Word not found.")
    } else {
        word_query.display(&word, &config);
        if let Err(err) = word_speech.await {
            eprintln!("An error occured in google speech module: {}.", err)
        }
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
 * 5. Better audio
 * 6. Better cache consistency with audio
 * 7. Cli
 */
