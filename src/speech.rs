use crate::Cache;
use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::File,
    io::{BufReader, Write},
    path::PathBuf,
};

/// Currently only a minimium set of langs are supported.
/// Considering adding more in the future.
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Lang {
    en_us,
    zh_cn,
}

impl Lang {
    pub fn new(word: impl AsRef<str>) -> Self {
        // worst way to distinguish language,
        // NOT ELEGANT :-(
        // but it works.
        if word.as_ref().is_ascii() {
            Lang::en_us
        } else {
            Lang::zh_cn
        }
    }
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lang::en_us => write!(f, "en_us"),
            Lang::zh_cn => write!(f, "zh_cn"),
        }
    }
}

pub struct Speech;

impl Speech {
    pub async fn spawn(
        word: String, _cache: Cache, is_speak: bool,
    ) -> anyhow::Result<()> {
            if is_speak {
                Speech::speak(word).await
            } else {
                Ok(())
            }
    }

    async fn speak(word: impl AsRef<str>) -> anyhow::Result<()> {
        let url = format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl={}&q={}",
            Lang::new(&word),
            word.as_ref()
        );
        let filename = PathBuf::from("./audio.mp3");

        // request
        let res = reqwest::get(url).await?;

        let mut file = File::create(&filename)?;
        let bytes = res.bytes().await?;
        file.write_all(&bytes)?;

        // rodio
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let file = BufReader::new(File::open(&filename)?);
        let source = Decoder::new(file)?;
        sink.append(source);
        sink.sleep_until_end();

        std::fs::remove_file(filename)?;
        Ok(())
    }
}
