mod authority;
mod sentence;
mod utils;

use self::{sentence::Sentence, utils::*};
use crate::Cache;
use scraper::{ElementRef, Html, Selector};
use serde::{Deserialize, Serialize};

pub trait Select {
    type Target;
    fn select(elem: ElementRef) -> anyhow::Result<Self::Target>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordQuery {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
}

impl WordQuery {
    pub fn is_empty(&self) -> bool {
        return self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty();
    }
}

pub struct WebQuery;

impl Select for WebQuery {
    type Target = WordQuery;

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let doc = elem;
        let pronunciation = {
            let sel = Selector::parse("span.pronounce").unwrap();
            doc.select(&sel)
                .filter_map(|child| {
                    let mut iter = child.text().filter_map(trim_str);
                    match (iter.next(), iter.next()) {
                        (Some(region), Some(pron)) => Some((region, pron)),
                        _ => None,
                    }
                })
                .collect()
        };

        let brief = {
            let sel = Selector::parse("#phrsListTab .trans-container ul li").unwrap();
            doc.select(&sel)
                .map(|child| {
                    child
                        .text()
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join("")
                })
                .collect()
        };

        let variants = {
            let sel = Selector::parse("#phrsListTab .trans-container p").unwrap();
            doc.select(&sel)
                .map(|child| {
                    child.text().map(|t| {
                        t.split("\n")
                            .filter_map(trim_str)
                            .collect::<Vec<String>>()
                            .join(" ")
                    })
                })
                .flatten()
                .collect()
        };

        let sentence = Sentence::select(elem)?;

        Ok(WordQuery {
            pronunciation,
            brief,
            variants,
            authority: Vec::new(),
            sentence,
        })
    }
}

impl WebQuery {
    pub fn new() -> Self {
        Self
    }
    pub async fn query(&mut self, query_word: impl AsRef<str>) -> anyhow::Result<WordQuery> {
        let youdao_dict_url = url::Url::parse(&format!(
            "http://dict.youdao.com/search?q={}",
            query_word.as_ref()
        ))?;

        let xml = get_html(youdao_dict_url).await?;
        let doc = Html::parse_document(&xml);

        WebQuery::select(doc.root_element())
    }
}

pub struct CacheQuery<'a> {
    cache: &'a mut Cache,
}

impl<'a> CacheQuery<'a> {
    pub fn new(cache: &'a mut Cache) -> Self {
        Self { cache }
    }
    pub async fn query(&mut self, query_word: impl AsRef<str>) -> anyhow::Result<WordQuery> {
        self.cache.query(query_word)
    }
}
