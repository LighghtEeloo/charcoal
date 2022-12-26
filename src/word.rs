pub mod display;
pub mod query;
pub mod select;
pub mod speech;

use self::query::Query;
use crate::Cache;
use serde::{Deserialize, Serialize};
use whatlang::Lang;

pub struct WordQuery {
    word: String,
    lang: Lang,
}

impl<'a> WordQuery {
    pub fn new(word: String) -> Option<Self> {
        if word.is_empty() {
            return None;
        }
        let lang = whatlang::detect(&word).map_or(Lang::Eng, |info| info.lang());
        Some(Self { word, lang })
    }
    pub fn word(&'a self) -> &'a str {
        &self.word
    }
    pub fn is_western(&self) -> bool {
        !matches!(self.lang, Lang::Cmn | Lang::Jpn)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordEntry {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
}

impl WordEntry {
    /// Query a word first from cache and then from the web
    pub async fn query(word_query: &WordQuery, cache: &Cache) -> anyhow::Result<Self> {
        (FromCache::new(cache).query(&word_query))
            .or_else(|_err| FromYoudict::new().query_and_store(word_query, cache))
    }

    pub fn is_empty(&self) -> bool {
        self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty()
    }
}

pub struct FromYoudict;

impl FromYoudict {
    pub fn new() -> Self {
        Self
    }
    pub fn query_and_store(
        &mut self, word_query: &WordQuery, cache: &Cache,
    ) -> anyhow::Result<WordEntry> {
        let word_entry = self.query(word_query)?;
        let file = cache.store(word_query.word(), "bin")?;
        bincode::serialize_into(file, &word_entry)?;
        Ok(word_entry)
    }
}

pub struct FromCache<'a> {
    cache: &'a Cache,
}

impl<'a> FromCache<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}
