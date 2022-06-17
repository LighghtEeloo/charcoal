use charcoal::{AppDataBuilder, Args, Speech, WordQuery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app_data_builder = AppDataBuilder::new();

    let mut config = app_data_builder.config()?;
    let mut cache = app_data_builder.cache()?;

    let args = Args::new();
    if args.speak {
        config.flip(charcoal::Toggle::WithSpeech)
    }

    let word = args.query;
    let speech = Speech::new(&config);

    let word_speech = speech.speak(&word);
    let word_query = {
        if let Ok(word_query) = cache.query(&word) {
            word_query
        } else {
            let word_query = WordQuery::query(&word).await?;
            cache.store(word.clone(), word_query.clone())?;
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
 * 1. Config
 * 4. Authority
 */
