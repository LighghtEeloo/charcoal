use crate::Cache;
use rodio::{Decoder, OutputStream, Sink};
use std::{
    fs::File,
    io::{BufReader, Write},
};
use whatlang::Lang;

pub struct Speech;

impl Speech {
    pub async fn query(
        word: impl AsRef<str>, lang: &Lang, cache: &Cache, is_speak: bool,
    ) -> anyhow::Result<()> {
        if is_speak {
            let file = Speech::store(word, lang, cache).await?;
            Speech::speak(file).await
        } else {
            Ok(())
        }
    }

    fn url(word: impl AsRef<str>, lang: &Lang) -> String {
        let code = match lang {
            Lang::Cmn | Lang::Jpn => "zh_cn",
            _ => "en",
        };
        format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl={}&q={}",
            code,
            word.as_ref()
        )
    }

    async fn store(word: impl AsRef<str>, lang: &Lang, cache: &Cache) -> anyhow::Result<File> {
        let file = (cache.query(&word, "mp3")).or_else(|_| -> anyhow::Result<File> {
            let url = Speech::url(&word, lang);
            futures::executor::block_on(async {
                // request
                let res = reqwest::get(url).await?;

                // write
                let mut file = cache.store(&word, "mp3")?;
                let bytes = res.bytes().await?;
                file.write_all(&bytes)?;

                // read again to avoid overflow
                let file = cache.query(&word, "mp3")?;
                Ok(file)
            })
        })?;
        Ok(file)
    }

    async fn speak(file: File) -> anyhow::Result<()> {
        // rodio
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        let file = BufReader::new(file);
        let source = Decoder::new(file)?;
        sink.append(source);
        sink.sleep_until_end();

        Ok(())
    }
}
