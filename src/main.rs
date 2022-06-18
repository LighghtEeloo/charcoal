use charcoal::{cli, AppDataBuilder, CacheQuery, Cli, Command, Speech, WebQuery, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app_data_builder = AppDataBuilder::new();

    let mut config = app_data_builder.config()?;
    let mut cache = app_data_builder.cache()?;

    let cli = Cli::new();
    let Command::Query(args) = cli.command;
    if let Some(speak) = args.speak {
        match speak {
            cli::Toggle::Y => config.speech = true,
            cli::Toggle::N => config.speech = false,
            cli::Toggle::T => Config::flip(&mut config.speech),
        }
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

/* TODO
 * 1. Config & Cli
 * 4. Authority
 */
