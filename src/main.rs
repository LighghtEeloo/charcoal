#![allow(dead_code)]

use charcoal::{cli, speak, Config, Word};
use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = cli::Args::parse();
    let config_path = "./charcoal.toml";

    let mut config = Config::of_path(config_path).unwrap_or_default();
    config.to_file(config_path)?;
    if args.speak {
        config.flip(charcoal::Toggle::WithSpeech)
    }

    // let word = "loom";
    // let word = "depreciate";
    let word = "jargon";

    let speech = speak(word, &config);
    let word = Word::query(word).await?;
    // println!("{:#?}", word);
    word.display(&config);

    if let Err(err) = speech.await {
        eprintln!("An error occured in google speech module: {}.", err)
    }

    Ok(())
}

/* TODO
 * 1. Clap
 * 3. Sentence
 * 4. Authority
 */
