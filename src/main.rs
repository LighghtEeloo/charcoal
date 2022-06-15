#![allow(dead_code)]

use charcoal::{speak, Config, Word};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let config = Config::all();

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
