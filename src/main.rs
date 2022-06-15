#![allow(dead_code)]

use scraper::{Html, Selector};

async fn get_html(url: impl AsRef<str> + reqwest::IntoUrl) -> anyhow::Result<String> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

#[derive(Clone, Debug)]
pub struct Word {
    word: String,
    pronunciation: Vec<(String, String)>,
    brief: Vec<String>,
    variants: Vec<String>,
    authority: Vec<String>,
    sentence: Vec<String>,
}

fn trim_str(t: &str) -> Option<String> {
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

async fn get_word(query_word: impl ToString) -> anyhow::Result<Word> {
    let word = query_word.to_string();
    let youdao_dict_url = url::Url::parse(&format!("http://dict.youdao.com/search?q={}", word))?;

    let xml = get_html(youdao_dict_url).await?;
    let doc = Html::parse_document(&xml);

    let pronunciation = {
        let mut vec = Vec::new();
        let sel = Selector::parse("span.pronounce").unwrap();
        for child in doc.select(&sel) {
            let pron = child.text().filter_map(trim_str).collect::<Vec<String>>();
            vec.push((pron[0].to_owned(), pron[1].to_owned()))
        }
        vec
    };
    let brief = {
        let mut vec = Vec::new();
        let sel = Selector::parse("#phrsListTab .trans-container ul li").unwrap();
        for child in doc.select(&sel) {
            vec.push(
                child
                    .text()
                    .filter_map(trim_str)
                    .collect::<Vec<String>>()
                    .join(""),
            );
        }
        vec
    };
    let variants = {
        let mut vec = Vec::new();
        let sel = Selector::parse("#phrsListTab .trans-container p").unwrap();
        for child in doc.select(&sel) {
            vec.extend(child.text().map(|t| {
                t.split("\n")
                    .filter_map(trim_str)
                    .collect::<Vec<String>>()
                    .join(" ")
            }));
        }
        vec
    };

    Ok(Word {
        word,
        pronunciation,
        brief,
        variants,
        authority: Vec::new(),
        sentence: Vec::new(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    // let word = "loom";
    // let word = "depreciate";
    let word = "jargon";

    let word = get_word(word).await?;
    println!("{:#?}", word);

    let speech = || -> anyhow::Result<()> {
        // let mut ttss = tts::Tts::default()?;
        // let tts::Features { is_speaking, .. } = ttss.supported_features();
        // if is_speaking {
        //     println!("Are we speaking? {}", ttss.is_speaking()?);
        // }
        // ttss.speak(word.word, false)?;
        let voice = google_speech::Speech::new(word.word, google_speech::Lang::en_us)?;
        voice.play()?;
        Ok(())
    };
    if let Err(err) = speech() {
        eprintln!("An error occured in google speech module: {}.", err)
    }

    Ok(())
}

/* TODO
 * 1. Clap
 * 2. Colorize
 * 3. Sentence
 * 4. Authority
 */
