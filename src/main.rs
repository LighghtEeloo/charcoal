#![allow(dead_code)]

use charcoal::{cli, speak, ConfigBuilder, Word};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let mut config = ConfigBuilder::build()?;
    let args = cli::Args::parse();

    if args.speak {
        config.flip(charcoal::Toggle::WithSpeech)
    }
    let query_word = args.query_word;

    let speech = speak(query_word.clone(), &config);
    let word = Word::query(query_word).await?;

    if word.is_empty() {
        println!("Word not found.")
    } else {
        word.display(&config);
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
