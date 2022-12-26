mod sentence;

use self::sentence::Sentence;
use super::{FromYoudict, Query, Select, WordEntry, WordQuery};
use scraper::{ElementRef, Selector};

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

impl Select for FromYoudict {
    type Target = WordEntry;

    fn select(elem: ElementRef, word_query: &WordQuery) -> anyhow::Result<Self::Target> {
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
                .flat_map(|child| {
                    child.text().map(|t| {
                        t.split('\n')
                            .filter_map(trim_str)
                            .collect::<Vec<String>>()
                            .join(" ")
                    })
                })
                .filter(|s| !s.is_empty())
                .collect()
        };

        let sentence = Sentence::select(elem, word_query)?;

        Ok(WordEntry {
            pronunciation,
            brief,
            variants,
            authority: Vec::new(),
            sentence,
        })
    }
}

fn trim_str(t: &str) -> Option<String> {
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}
