use super::{select::Select, FromCache, FromYoudict, WordEntry, WordQuery};

pub trait Query {
    fn query(&mut self, word_query: &WordQuery) -> anyhow::Result<WordEntry>;
}

impl Query for FromYoudict {
    fn query(&mut self, word_query: &WordQuery) -> anyhow::Result<WordEntry> {
        async fn get_html(url: impl AsRef<str> + reqwest::IntoUrl) -> anyhow::Result<String> {
            let body = reqwest::get(url).await?.text().await?;
            Ok(body)
        }
        let youdao_dict_url = url::Url::parse(&format!(
            "http://dict.youdao.com/search?q={}",
            word_query.word()
        ))?;

        let xml = futures::executor::block_on(async { get_html(youdao_dict_url).await })?;
        let doc = scraper::Html::parse_document(&xml);

        FromYoudict::select(doc.root_element(), word_query)
    }
}

impl<'a> Query for FromCache<'a> {
    fn query(&mut self, word_query: &WordQuery) -> anyhow::Result<WordEntry> {
        let file = self.cache.query(word_query.word(), "bin")?;
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
}
