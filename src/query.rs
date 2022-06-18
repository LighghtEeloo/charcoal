use crate::{Cache, Select, WordEntry};
use scraper::Html;

pub struct FromYoudict;

impl FromYoudict {
    pub fn new() -> Self {
        Self
    }
    pub async fn query(&mut self, query_word: impl AsRef<str>) -> anyhow::Result<WordEntry> {
        async fn get_html(url: impl AsRef<str> + reqwest::IntoUrl) -> anyhow::Result<String> {
            let body = reqwest::get(url).await?.text().await?;
            Ok(body)
        }
        let youdao_dict_url = url::Url::parse(&format!(
            "http://dict.youdao.com/search?q={}",
            query_word.as_ref()
        ))?;

        let xml = get_html(youdao_dict_url).await?;
        let doc = Html::parse_document(&xml);

        FromYoudict::select(doc.root_element())
    }
}

pub struct FromCache<'a> {
    cache: &'a mut Cache,
}

impl<'a> FromCache<'a> {
    pub fn new(cache: &'a mut Cache) -> Self {
        Self { cache }
    }
    pub async fn query(&mut self, query_word: impl AsRef<str>) -> anyhow::Result<WordEntry> {
        self.cache.query(query_word)
    }
}
