pub mod cached;
pub mod pprint;
pub mod speech;
pub mod youdict;

use crate::{Cache, Config};
use scraper::{ElementRef, Html};
use serde::{Deserialize, Serialize};
use whatlang::Lang;

pub struct ExactQuery {
    word: String,
    lang: Lang,
}

impl<'a> ExactQuery {
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
pub struct SingleEntry {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
}

impl SingleEntry {
    /// Query a word first from cache and then from the web
    pub async fn query(word_query: &ExactQuery, cache: &Cache) -> anyhow::Result<Self> {
        (QueryCache::new(cache).query(&word_query))
            .or_else(|_err| QueryYoudict::new().query_and_store(word_query, cache))
    }

    pub fn is_empty(&self) -> bool {
        self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty()
    }
}

pub trait Query {
    type WordQuery;
    type WordEntry;
    fn query(&mut self, word_query: &Self::WordQuery) -> anyhow::Result<Self::WordEntry>;
}

pub trait Request {
    type WordQuery;
    fn request(&mut self, word_query: &Self::WordQuery) -> anyhow::Result<Html>;
}

pub trait Select {
    type WordQuery;
    type Target;
    fn select(elem: ElementRef, word_query: &Self::WordQuery) -> anyhow::Result<Self::Target>;
}

pub trait PPrint {
    type WordQuery;
    fn pprint(&self, word_query: &Self::WordQuery, config: &Config);
}

pub struct QueryYoudict;

impl QueryYoudict {
    pub fn new() -> Self {
        Self
    }
    pub fn query_and_store(
        &mut self, word_query: &ExactQuery, cache: &Cache,
    ) -> anyhow::Result<SingleEntry> {
        let word_entry = self.query(word_query)?;
        let file = cache.store(word_query.word(), "bin")?;
        bincode::serialize_into(file, &word_entry)?;
        Ok(word_entry)
    }
}

pub struct QueryCache<'a> {
    cache: &'a Cache,
}

impl<'a> QueryCache<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}
