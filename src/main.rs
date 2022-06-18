use charcoal::{cli, AppDataBuilder, CacheQuery, Cli, Command, Speech, WebQuery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    match Cli::new() {
        Command::Query(args) => query_main(args).await,
        Command::Edit => edit_main().await,
    }
}

async fn query_main(args: cli::QueryArgs) -> anyhow::Result<()> {
    let app_data_builder = AppDataBuilder::new();

    let mut config = app_data_builder.config()?;
    let mut cache = app_data_builder.cache()?;

    if let Some(speak) = args.speak {
        speak.twitch(&mut config.speak)
    }

    let word = args.query;
    let speech = Speech::new(&config);

    let word_speech = speech.speak(&word);
    let word_query = {
        if let Ok(word_query) = CacheQuery::new(&mut cache).query(&word).await {
            word_query
        } else {
            let word_query = WebQuery::new().query(&word).await?;
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

async fn edit_main() -> anyhow::Result<()> {
    let editor = std::env!("EDITOR");
    let config_path = AppDataBuilder::new().config()?.path;

    let mut child = std::process::Command::new(editor).args([config_path]).spawn()?;
    child.wait()?;
    Ok(())
}

/* TODO
 * 1. Config & Cli
 * 4. Authority
 * 5. Better audio
 * 6. Better cache consistency with audio
 */
