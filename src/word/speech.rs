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
        if let Some(audio) = Self::prepare(word_query, cache, is_speak).await? {
            Self::speak(audio).await?;
        }
        Ok(())
    }

    pub(crate) async fn prepare(
        query: &impl Question, cache: &Cache, enabled: bool,
    ) -> anyhow::Result<Option<Cursor<Vec<u8>>>> {
        if enabled {
            Ok(Some(Self::store(query, cache).await?))
        } else {
            Ok(None)
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
        Self::store_with_fetch(word_query, cache, async {
            crate::word::http::get(&crate::word::http::client()?, Self::url(word_query)?).await
        })
        .await
    }

    async fn store_with_fetch(
        word_query: &impl Question, cache: &Cache,
        fetch: impl std::future::Future<Output = anyhow::Result<Vec<u8>>>,
    ) -> anyhow::Result<Cursor<Vec<u8>>> {
        let key = CacheKey::new(&word_query.word(), word_query.lang(), "google-tts", 1);
        if !word_query.refresh() {
            let cached_key = key.clone();
            let cached_cache = cache.clone();
            let cached = move || -> anyhow::Result<Vec<u8>> {
                let mut bytes = Vec::new();
                cached_cache
                    .query(&cached_key, "mp3")?
                    .read_to_end(&mut bytes)?;
                Self::validate(&bytes)?;
                Ok(bytes)
            };
            match tokio::task::spawn_blocking(cached)
                .await
                .map_err(anyhow::Error::from)
                .and_then(|result| result)
            {
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
        let bytes = fetch.await?;
        let cache = cache.clone();
        tokio::task::spawn_blocking(move || {
            Self::validate(&bytes)?;
            if let Err(err) = cache.store(&key, "mp3", &bytes) {
                log::warn!("Failed to cache speech audio: {err}");
            }
            Ok(Cursor::new(bytes))
        })
        .await
        .context("Speech cache task failed")?
    }

    pub(crate) async fn speak(file: Cursor<Vec<u8>>) -> anyhow::Result<()> {
        tokio::task::spawn_blocking(move || Self::play_blocking(file))
            .await
            .context("Speech playback task failed")?
    }

    fn play_blocking(file: Cursor<Vec<u8>>) -> anyhow::Result<()> {
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

    async fn store_at(
        query: &impl Question, cache: &Cache, url: url::Url,
    ) -> anyhow::Result<Cursor<Vec<u8>>> {
        Speech::store_with_fetch(
            query,
            cache,
            crate::word::http::get(&crate::word::http::tests::client(), url),
        )
        .await
    }

    fn wav() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&68u32.to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&8000u32.to_le_bytes());
        bytes.extend_from_slice(&16000u32.to_le_bytes());
        bytes.extend_from_slice(&2u16.to_le_bytes());
        bytes.extend_from_slice(&16u16.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&32u32.to_le_bytes());
        bytes.extend_from_slice(&[0; 32]);
        bytes
    }

    fn cache(root: &std::path::Path) -> Cache {
        Cache::new(root.join("cache"), root.join("vault"), root.join("tmp"))
    }

    #[tokio::test]
    async fn valid_audio_cache_works_without_network_or_playback() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path());
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let key = CacheKey::new("hello", Lang::Eng, "google-tts", 1);
        cache.store(&key, "mp3", &wav()).unwrap();
        let result = store_at(
            &query,
            &cache,
            url::Url::parse("http://127.0.0.1:1").unwrap(),
        )
        .await
        .unwrap();
        assert_eq!(result.into_inner(), wav());
        assert!(
            Speech::prepare(&query, &cache, false)
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn corrupt_audio_is_replaced_and_refresh_bypasses_valid_cache() {
        for refresh in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let cache = cache(root.path());
            let query = ExactQuery::new("hello".into(), Lang::Eng, refresh).unwrap();
            let key = CacheKey::new("hello", Lang::Eng, "google-tts", 1);
            let initial = if refresh {
                let mut audio = wav();
                audio[44] = 1;
                audio
            } else {
                b"invalid audio".to_vec()
            };
            cache.store(&key, "mp3", &initial).unwrap();
            let (url, server) =
                crate::word::http::tests::server_with_body(200, std::time::Duration::ZERO, wav())
                    .await;
            let result = store_at(&query, &cache, url).await.unwrap();
            assert_eq!(result.into_inner(), wav());
            let mut cached = Vec::new();
            cache
                .query(&key, "mp3")
                .unwrap()
                .read_to_end(&mut cached)
                .unwrap();
            assert_eq!(cached, wav());
            server.finish().await.unwrap();
        }
    }

    #[tokio::test]
    async fn invalid_audio_responses_are_never_cached() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path());
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let (url, server) = crate::word::http::tests::server_with_body(
            200,
            std::time::Duration::ZERO,
            "<html>Verification required</html>",
        )
        .await;
        let error = store_at(&query, &cache, url).await.unwrap_err();
        assert!(
            error.is::<rodio::decoder::DecoderError>(),
            "Expected invalid audio, got: {error:#}"
        );
        let key = CacheKey::new("hello", Lang::Eng, "google-tts", 1);
        assert!(cache.query(&key, "mp3").is_err());
        server.finish().await.unwrap();
    }

    #[tokio::test]
    async fn downloaded_audio_survives_cache_write_failure() {
        let root = tempfile::tempdir().unwrap();
        let blocked = root.path().join("blocked");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let cache = Cache::new(blocked, root.path().join("vault"), root.path().join("tmp"));
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let (url, server) =
            crate::word::http::tests::server_with_body(200, std::time::Duration::ZERO, wav()).await;
        assert_eq!(
            store_at(&query, &cache, url).await.unwrap().into_inner(),
            wav()
        );
        server.finish().await.unwrap();
    }

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
