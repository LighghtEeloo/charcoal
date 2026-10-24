mod collins;
mod youdict;

use crate::app::cache::CacheKey;
use crate::{Acquire, Answer, Cache, ExactQuery, Question, SingleEntry};
use anyhow::Context;

/// Bump when the parsed dictionary entry representation changes so that entries
/// written by older parsers are not served from cache. Version 2 skips the empty
/// example rows produced for Youdao's audio-only media examples; version 3 adds
/// the parsed Collins dictionary block.
const DICTIONARY_CACHE_FORMAT_VERSION: u32 = 3;

pub struct QueryCache<'a> {
    cache: &'a Cache,
}

impl<'a> QueryCache<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Acquire for QueryCache<'a> {
    type WordQuery = ExactQuery;
    type WordEntry = SingleEntry;
    fn acquire(self, word_query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        if word_query.refresh {
            anyhow::bail!("Force refreshing the cache.")
        }
        let mut file = self.cache.query(&word_query.cache_key(), "bin")?;
        let mut buf = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut buf)?;
        let entry: SingleEntry =
            wincode::deserialize_from(buf.as_slice()).context("Invalid dictionary cache data")?;
        anyhow::ensure!(!entry.not_found(), "Ignoring empty dictionary cache entry");
        Ok(entry)
    }
}

pub struct QueryYoudict {
    endpoint: url::Url,
}

impl QueryYoudict {
    pub fn new() -> Self {
        Self {
            endpoint: url::Url::parse("https://dict.youdao.com/search")
                .expect("Valid dictionary endpoint"),
        }
    }
    pub async fn query_and_store(
        self, word_query: &ExactQuery, cache: &Cache,
    ) -> anyhow::Result<SingleEntry> {
        self.query_and_store_with_client(word_query, cache, &crate::word::http::client()?)
            .await
    }

    async fn query_and_store_with_client(
        self, word_query: &ExactQuery, cache: &Cache, client: &reqwest::Client,
    ) -> anyhow::Result<SingleEntry> {
        let word_entry = self.acquire_with_client(word_query, client).await?;
        let query = word_query.clone();
        let cache = cache.clone();
        let entry = word_entry.clone();
        if let Err(err) =
            tokio::task::spawn_blocking(move || Self::cache_entry(&query, &cache, &entry)).await
        {
            log::warn!("Dictionary cache task failed: {err}");
        }

        Ok(word_entry)
    }

    fn cache_entry(query: &ExactQuery, cache: &Cache, entry: &SingleEntry) {
        if entry.not_found() {
            return;
        }
        let write = || -> anyhow::Result<()> {
            let mut buf = Vec::new();
            wincode::serialize_into(&mut buf, entry)?;
            cache.store(&query.cache_key(), "bin", &buf)?;
            Ok(())
        };
        if let Err(err) = write() {
            log::warn!("Failed to cache dictionary result: {err:#}");
        }
    }
}

impl ExactQuery {
    pub(crate) fn cache_key(&self) -> CacheKey {
        CacheKey::new(
            &self.word(),
            self.lang(),
            "youdao",
            DICTIONARY_CACHE_FORMAT_VERSION,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use whatlang::Lang;

    fn entry() -> SingleEntry {
        SingleEntry {
            pronunciation: vec![],
            brief: vec!["A greeting".into()],
            variants: vec![],
            authority: vec![],
            sentence: vec![],
            collins: None,
        }
    }

    #[test]
    fn cache_write_failures_do_not_discard_results() {
        let root = tempfile::tempdir().unwrap();
        let blocked = root.path().join("blocked");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let cache = Cache::new(blocked, root.path().join("vault"), root.path().join("tmp"));
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let entry = entry();
        QueryYoudict::cache_entry(&query, &cache, &entry);
        assert_eq!(entry.brief, ["A greeting"]);
        assert!(cache.query(&query.cache_key(), "bin").is_err());
    }

    #[test]
    fn empty_results_are_not_cached() {
        let root = tempfile::tempdir().unwrap();
        let cache = Cache::new(
            root.path().join("cache"),
            root.path().join("vault"),
            root.path().join("tmp"),
        );
        let query = ExactQuery::new("unknown".into(), Lang::Eng, false).unwrap();
        let mut entry = entry();
        entry.brief.clear();
        QueryYoudict::cache_entry(&query, &cache, &entry);
        assert_eq!(
            cache.query(&query.cache_key(), "bin").unwrap_err().kind(),
            std::io::ErrorKind::NotFound
        );
    }

    #[test]
    fn corrupt_and_empty_cache_entries_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let cache = Cache::new(
            root.path().join("cache"),
            root.path().join("vault"),
            root.path().join("tmp"),
        );
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        cache.store(&query.cache_key(), "bin", b"invalid").unwrap();
        assert!(QueryCache::new(&cache).acquire(&query).is_err());
        let mut entry = entry();
        entry.brief.clear();
        let mut bytes = Vec::new();
        wincode::serialize_into(&mut bytes, &entry).unwrap();
        cache.store(&query.cache_key(), "bin", &bytes).unwrap();
        assert!(QueryCache::new(&cache).acquire(&query).is_err());
    }

    #[test]
    fn entries_from_the_previous_cache_format_version_are_ignored() {
        let root = tempfile::tempdir().unwrap();
        let cache = Cache::new(
            root.path().join("cache"),
            root.path().join("vault"),
            root.path().join("tmp"),
        );
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let mut bytes = Vec::new();
        wincode::serialize_into(&mut bytes, &entry()).unwrap();

        // Entries written by the previous parser carry a different `SingleEntry`
        // layout (they have no Collins block), so they must not be served.
        let previous = CacheKey::new(
            &query.word(),
            query.lang(),
            "youdao",
            DICTIONARY_CACHE_FORMAT_VERSION - 1,
        );
        cache.store(&previous, "bin", &bytes).unwrap();
        assert!(QueryCache::new(&cache).acquire(&query).is_err());

        // Entries written with the current format version keep round-tripping.
        cache.store(&query.cache_key(), "bin", &bytes).unwrap();
        assert_eq!(
            QueryCache::new(&cache).acquire(&query).unwrap().brief,
            ["A greeting"]
        );
    }

    #[test]
    fn collins_entries_round_trip_through_the_cache() {
        use crate::word::frontend::{CollinsEntry, CollinsGroup, CollinsSense};

        let root = tempfile::tempdir().unwrap();
        let cache = Cache::new(
            root.path().join("cache"),
            root.path().join("vault"),
            root.path().join("tmp"),
        );
        let query = ExactQuery::new("fast".into(), Lang::Eng, false).unwrap();
        let mut entry = entry();
        entry.collins = Some(CollinsEntry {
            groups: vec![CollinsGroup {
                title: Some("VERB USES".into()),
                headword: "fast".into(),
                phonetic: Some("/fæst/".into()),
                ranks: Some("CET4 TEM4".into()),
                word_forms: Some("(fasting, fasted, fasts)".into()),
                senses: vec![CollinsSense {
                    pos: Some("V-I".into()),
                    definition: "If you fast, you eat nothing.".into(),
                    translation: "禁食".into(),
                    examples: vec![("She fasted for three days.".into(), "她禁食了三天。".into())],
                }],
            }],
        });

        QueryYoudict::cache_entry(&query, &cache, &entry);
        let loaded = QueryCache::new(&cache).acquire(&query).unwrap();
        assert_eq!(loaded.collins, entry.collins);
    }
}
