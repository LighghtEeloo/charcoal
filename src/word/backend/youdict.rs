use crate::word::{Acquire, QueryYoudict, Request, Select};
use crate::{ExactQuery, Question, SingleEntry};
use scraper::{ElementRef, Html, Selector};
use whatlang::Lang;

impl Acquire for QueryYoudict {
    type WordQuery = ExactQuery;
    type WordEntry = SingleEntry;
    fn acquire(self, word_query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        let doc = self.request(word_query)?;
        QueryYoudict::select(doc.root_element(), word_query)
    }
}

impl Request for QueryYoudict {
    type WordQuery = ExactQuery;
    fn request(self, word_query: &ExactQuery) -> anyhow::Result<Html> {
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

        Ok(doc)
    }
}

impl Select for QueryYoudict {
    type WordQuery = ExactQuery;
    type Target = SingleEntry;

    fn select(elem: ElementRef, word_query: &ExactQuery) -> anyhow::Result<Self::Target> {
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

        Ok(SingleEntry {
            pronunciation,
            brief,
            variants,
            authority: Vec::new(),
            sentence,
        })
    }
}

pub struct Sentence;

impl Select for Sentence {
    type WordQuery = ExactQuery;
    type Target = Vec<(String, String)>;

    fn select(elem: ElementRef, word_query: &ExactQuery) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("#bilingual.trans-container li").unwrap();
        Ok(elem
            .select(&sel)
            .filter_map(|child| Sen::select(child, word_query).ok())
            .collect())
    }
}

struct Sen;
const PUNCTUATORS: &[char; 10] = &['.', ',', '\"', '\'', '?', '!', ':', '-', '<', '>'];

impl Select for Sen {
    type WordQuery = ExactQuery;
    type Target = (String, String);

    fn select(elem: ElementRef, word_query: &ExactQuery) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("p").unwrap();
        let mut iter = elem.select(&sel);

        let mut extract_to_vec = |msg| -> Vec<String> {
            iter.next()
                .expect(msg)
                .text()
                .filter_map(trim_str)
                .collect()
        };

        fn western_concat(vec: Vec<String>) -> String {
            let mut ori = String::new();
            let mut ori_iter = vec.into_iter();
            if let Some(s) = ori_iter.next() {
                ori.push_str(&s)
            }
            for mut s in ori_iter {
                if !s.starts_with(PUNCTUATORS) {
                    s.insert(0, ' ')
                }
                ori.push_str(&s)
            }
            ori
        }

        fn eastern_concat(vec: Vec<String>) -> String {
            vec.join("")
        }

        let ori_vec = extract_to_vec("No ori found in sentence");
        let trans_vec = extract_to_vec("No trans found in sentence");

        let (ori, trans) = if matches!(
            word_query.lang(),
            Lang::Cmn | Lang::Jpn | Lang::Kor
        ) {
            (eastern_concat(ori_vec), western_concat(trans_vec))
        } else {
            (western_concat(ori_vec), eastern_concat(trans_vec))
        };
        Ok((ori, trans))
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
