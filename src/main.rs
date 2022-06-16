#![allow(dead_code)]

use charcoal::{cli, speak, ConfigBuilder, WordQuery};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut config = ConfigBuilder::new().build()?;
    let args = cli::Args::parse();

    if args.speak {
        config.flip(charcoal::Toggle::WithSpeech)
    }
    let word = args.query_word;

    let speech = speak(&word, &config);
    let word_query = WordQuery::query(&word).await?;

    if word_query.is_empty() {
        println!("Word not found.")
    } else {
        word_query.display(&word, &config);
        if let Err(err) = speech.await {
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
