pub mod display;
pub mod select;
pub mod speech;

use crate::{Cache, Select};
use serde::{Deserialize, Serialize};
use whatlang::Lang;

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
    pub async fn query(word: impl AsRef<str>, lang: &Lang, cache: &Cache) -> anyhow::Result<Self> {
        (FromCache::new(cache).query(&word).await)
            .or_else(|_err| FromYoudict::new().query_and_store(word, lang, cache))
    }

    pub fn is_empty(&self) -> bool {
        return self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty();
    }
}

pub struct FromYoudict;

impl FromYoudict {
    pub fn new() -> Self {
        Self
    }
    pub fn query_and_store(
        &mut self, word: impl AsRef<str>, lang: &Lang, cache: &Cache,
    ) -> anyhow::Result<WordEntry> {
        futures::executor::block_on(async {
            let word_entry = self.query(&word, lang).await?;
            let file = cache.store(&word, "bin")?;
            bincode::serialize_into(file, &word_entry)?;
            Ok(word_entry)
        })
    }
    pub async fn query(&mut self, word: impl AsRef<str>, lang: &Lang) -> anyhow::Result<WordEntry> {
        async fn get_html(url: impl AsRef<str> + reqwest::IntoUrl) -> anyhow::Result<String> {
            let body = reqwest::get(url).await?.text().await?;
            Ok(body)
        }
        let youdao_dict_url = url::Url::parse(&format!(
            "http://dict.youdao.com/search?q={}",
            word.as_ref()
        ))?;

        let xml = get_html(youdao_dict_url).await?;
        let doc = scraper::Html::parse_document(&xml);

        FromYoudict::select(doc.root_element(), lang)
    }
}

pub struct FromCache<'a> {
    cache: &'a Cache,
}

impl<'a> FromCache<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
    pub async fn query(&mut self, query_word: impl AsRef<str>) -> anyhow::Result<WordEntry> {
        let file = self.cache.query(query_word, "bin")?;
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
}
