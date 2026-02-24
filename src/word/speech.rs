use crate::{Cache, Question};
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::{fs::File, io::Write};
use whatlang::Lang;

pub struct Speech;

impl Speech {
    pub async fn query(
        word_query: &impl Question, cache: &Cache, is_speak: bool,
    ) -> anyhow::Result<()> {
        if is_speak {
            let file = Speech::store(word_query, cache).await?;
            Speech::speak(file).await
        } else {
            Ok(())
        }
    }

    fn url(word_query: &impl Question) -> anyhow::Result<String> {
        let lang = word_query.lang();
        let code = match lang {
            Lang::Eng => "en",
            Lang::Fra => "fr",
            Lang::Cmn => "zh_cn",
            // Fixme: add more languages
            _ => "en",
            // _ => Err(anyhow::anyhow!("Language inferred ({}) not supported", lang))?,
        };
        Ok(format!(
            "https://translate.google.com/translate_tts?ie=UTF-8&client=tw-ob&tl={}&q={}",
            code,
            word_query.word()
        ))
    }

    async fn store(word_query: &impl Question, cache: &Cache) -> anyhow::Result<File> {
        let word = word_query.word();
        let file = (cache.query(&word, "mp3")).or_else(|_| -> anyhow::Result<File> {
            let url = Speech::url(word_query);
            futures::executor::block_on(async {
                // request
                let res = reqwest::get(url?).await?;

                // write
                let mut file = cache.store(&word, "mp3")?;
                let bytes = res.bytes().await?;
                file.write_all(&bytes)?;

                // read again to avoid overflow
                let file = cache.query(word, "mp3")?;
                Ok(file)
            })
        })?;
        Ok(file)
    }

    async fn speak(file: File) -> anyhow::Result<()> {
        let handle = DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(handle.mixer());
        let source = Decoder::try_from(file)?;
        player.append(source);
        player.sleep_until_end();
        Ok(())
    }
}
