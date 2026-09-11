use crate::word::{Acquire, QueryYoudict, Request, Select};
use crate::{ExactQuery, Question, SingleEntry};
use scraper::{ElementRef, Html, Selector};

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

impl Select for Sen {
    type WordQuery = ExactQuery;
    type Target = (String, String);

    fn select(elem: ElementRef, _word_query: &ExactQuery) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("p").unwrap();
        let mut iter = elem.select(&sel);

        let mut extract = |msg| -> anyhow::Result<String> {
            let paragraph = iter.next().ok_or_else(|| anyhow::anyhow!("{msg}"))?;
            // Inline elements may split words or contain the spaces between them.
            // Preserve those boundaries before collapsing HTML whitespace.
            Ok(paragraph
                .text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" "))
        };

        let ori = extract("No ori found in sentence")?;
        let trans = extract("No trans found in sentence")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use whatlang::Lang;

    #[test]
    fn chinese_examples_preserve_spacing_regardless_of_configured_language() {
        let doc = Html::parse_document(include_str!("fixtures/qingxi.html"));
        let expected = vec![
            (
                "从空中看去，那座庙宇清晰可辨。".to_owned(),
                "The temple was clearly visible from the air.".to_owned(),
            ),
            (
                "她给了我清晰而准确的指示。".to_owned(),
                "She gave me clear and precise directions.".to_owned(),
            ),
            (
                "电话里的声音清晰洪亮。".to_owned(),
                "The voice on the phone was clear and strong.".to_owned(),
            ),
        ];

        for lang in [Lang::Eng, Lang::Cmn, Lang::Jpn, Lang::Kor] {
            let query = ExactQuery::new("清晰".to_owned(), lang, false).unwrap();
            let entry = QueryYoudict::select(doc.root_element(), &query).unwrap();
            assert_eq!(entry.sentence, expected);
        }
    }

    #[test]
    fn english_examples_preserve_inline_words_punctuation_and_mixed_text() {
        let doc = Html::parse_fragment(
            r#"<li>
                <p>
                    The <b>clear</b>est <span>sign</span>:&#9; "<b>你好</b>" (it's <b>clear</b>).
                </p>
                <p>最<b>清晰</b>的标志：<span>Hello world</span>。</p>
            </li>"#,
        );
        let query = ExactQuery::new("clear".to_owned(), Lang::Eng, false).unwrap();
        let pair = Sen::select(doc.root_element(), &query).unwrap();
        assert_eq!(pair.0, "The clearest sign: \"你好\" (it's clear).");
        assert_eq!(pair.1, "最清晰的标志：Hello world。");
    }

    #[test]
    fn incomplete_examples_are_skipped() {
        let doc = Html::parse_fragment(
            r#"<div id="bilingual" class="trans-container"><li><p>Incomplete</p></li></div>"#,
        );
        let query = ExactQuery::new("clear".to_owned(), Lang::Eng, false).unwrap();
        assert!(
            Sentence::select(doc.root_element(), &query)
                .unwrap()
                .is_empty()
        );
    }
}
