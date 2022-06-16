#![allow(dead_code)]

use charcoal::{Args, ConfigBuilder, Speech, WordQuery};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut config = ConfigBuilder::new().build()?;
    let args = Args::new();

    if args.speak {
        config.flip(charcoal::Toggle::WithSpeech)
    }
    let word = args.query;
    let speech = Speech::new(&config);

    let word_speech = speech.speak(&word);
    let word_query = WordQuery::query(&word).await?;

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
 * 2. Cache
 * 4. Authority
 */
