use crate::app::cache::CacheKey;
use crate::{Cache, Question};
use anyhow::Context;
use rodio::{Decoder, DeviceSinkBuilder, Player};
use std::io::{Cursor, Read};
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

    fn url(word_query: &impl Question) -> anyhow::Result<url::Url> {
        let code = match word_query.lang() {
            Lang::Eng => "en",
            Lang::Fra => "fr",
            Lang::Cmn => "zh_cn",
            _ => "en",
        };
        let mut url = url::Url::parse("https://translate.google.com/translate_tts")?;
        url.query_pairs_mut().extend_pairs([
            ("ie", "UTF-8"),
            ("client", "tw-ob"),
            ("tl", code),
            ("q", &word_query.word()),
        ]);
        Ok(url)
    }

    fn validate(bytes: &[u8]) -> anyhow::Result<()> {
        Decoder::try_from(Cursor::new(bytes.to_vec())).context("Invalid speech audio")?;
        Ok(())
    }

    async fn store(word_query: &impl Question, cache: &Cache) -> anyhow::Result<Cursor<Vec<u8>>> {
        let key = CacheKey::new(&word_query.word(), word_query.lang(), "google-tts", 1);
        if !word_query.refresh() {
            let cached = || -> anyhow::Result<Vec<u8>> {
                let mut bytes = Vec::new();
                cache.query(&key, "mp3")?.read_to_end(&mut bytes)?;
                Self::validate(&bytes)?;
                Ok(bytes)
            };
            match cached() {
                Ok(bytes) => return Ok(Cursor::new(bytes)),
                Err(err) => {
                    if !err
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|err| err.kind() == std::io::ErrorKind::NotFound)
                    {
                        log::warn!("Ignoring unavailable speech cache: {err:#}");
                    }
                }
            }
        }
        let bytes = futures::executor::block_on(crate::word::http::get(
            &crate::word::http::client()?,
            Self::url(word_query)?,
        ))?;
        Self::validate(&bytes)?;
        if let Err(err) = cache.store(&key, "mp3", &bytes) {
            log::warn!("Failed to cache speech audio: {err}");
        }
        Ok(Cursor::new(bytes))
    }

    async fn speak(file: Cursor<Vec<u8>>) -> anyhow::Result<()> {
        let mut handle = DeviceSinkBuilder::open_default_sink()?;
        handle.log_on_drop(false);
        let player = Player::connect_new(handle.mixer());
        let source = Decoder::try_from(file)?;
        player.append(source);
        player.sleep_until_end();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExactQuery;

    #[test]
    fn speech_parameters_are_encoded_and_language_is_preserved() {
        let query = ExactQuery::new("C++ & x=1#tail".into(), Lang::Fra, false).unwrap();
        let url = Speech::url(&query).unwrap();
        let pairs: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(pairs["q"], "C++ & x=1#tail");
        assert_eq!(pairs["tl"], "fr");
        assert!(url.fragment().is_none());
    }

    #[test]
    fn html_responses_are_not_audio() {
        assert!(Speech::validate(b"<html>Please verify your request</html>").is_err());
    }
}
