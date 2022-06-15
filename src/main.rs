#![allow(dead_code)]

use charcoal::{speak, Word};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let word = "loom";
    // let word = "depreciate";
    let word = "jargon";

    let speech = speak(word);
    let word = Word::query(word).await?;
    // println!("{:#?}", word);
    println!("{}", word);

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
