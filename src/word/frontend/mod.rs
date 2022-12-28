use crate::{Acquire, Answer, Cache, Question};
use serde::{Deserialize, Serialize};
use whatlang::Lang;

use super::{QueryCache, QueryYoudict};

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
}

impl Question for ExactQuery {
    fn word(&self) -> String {
        self.word.clone()
    }
    fn assumed_lang(&self) -> Lang {
        self.lang
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
        (QueryCache::new(cache).acquire(word_query))
            .or_else(|_err| QueryYoudict::new().query_and_store(word_query, cache))
    }
}

impl Answer for SingleEntry {
    fn not_found(&self) -> bool {
        self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty()
    }
}
