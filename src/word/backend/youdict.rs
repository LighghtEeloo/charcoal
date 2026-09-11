use crate::word::{QueryYoudict, Request, Select};
use crate::{Answer, ExactQuery, Question, SingleEntry};
use scraper::{ElementRef, Html, Selector};

impl QueryYoudict {
    pub async fn acquire(self, word_query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        let doc = self.request(word_query).await?;
        QueryYoudict::parse_document(&doc, word_query)
    }
}

impl Request for QueryYoudict {
    type WordQuery = ExactQuery;
    async fn request(self, word_query: &ExactQuery) -> anyhow::Result<Html> {
        let url = self.url(word_query)?;
        let bytes = crate::word::http::get(&crate::word::http::client()?, url).await?;
        let text = String::from_utf8(bytes)?;
        let doc = Html::parse_document(&text);

        Ok(doc)
    }
}

#[derive(Debug)]
pub struct UnexpectedPage;

impl std::fmt::Display for UnexpectedPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "Unrecognized dictionary response: the page may have changed or require verification",
        )
    }
}
impl std::error::Error for UnexpectedPage {}

impl QueryYoudict {
    fn url(&self, query: &ExactQuery) -> anyhow::Result<url::Url> {
        let mut url = self.endpoint.clone();
        url.query_pairs_mut().append_pair("q", &query.word());
        Ok(url)
    }

    fn parse_document(doc: &Html, query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        let entry = Self::select(doc.root_element(), query)?;
        if entry.not_found() {
            // Observed no-result pages contain only the article placeholder and scripts.
            // Unknown dictionary sections must not be mistaken for missing words.
            let results = Selector::parse("#results-contents").unwrap();
            let known_empty = doc.select(&results).next().is_some_and(|root| {
                let mut article = false;
                for child in root.child_elements() {
                    match (child.value().name(), child.value().attr("id")) {
                        (_, Some("wordArticle")) => article = true,
                        ("script", _) => {}
                        _ => return false,
                    }
                }
                article
            });
            if !known_empty {
                return Err(UnexpectedPage.into());
            }
        }
        Ok(entry)
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

    #[tokio::test]
    async fn lookup_runs_on_a_single_thread_runtime_and_caches_results() {
        let (url, server) = crate::word::http::tests::server_with_body(
            200, std::time::Duration::ZERO,
            "<div id='phrsListTab'><div class='trans-container'><ul><li>A greeting</li></ul></div></div>",
        ).await;
        let root = tempfile::tempdir().unwrap();
        let cache = crate::Cache::new(
            root.path().join("cache"),
            root.path().join("vault"),
            root.path().join("tmp"),
        );
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let entry = QueryYoudict { endpoint: url }
            .query_and_store(&query, &cache)
            .await
            .unwrap();
        assert_eq!(entry.brief, ["A greeting"]);
        assert!(cache.query(&query.cache_key(), "bin").is_ok());
        server.await.unwrap();
    }

    #[tokio::test]
    async fn failed_and_empty_remote_results_never_create_cache_entries() {
        for (status, body, is_error) in [
            (500, "Service unavailable", true),
            (200, "<html>Verification required</html>", true),
            (
                200,
                "<div id='results-contents'><div id='wordArticle'></div></div>",
                false,
            ),
        ] {
            let (url, server) =
                crate::word::http::tests::server_with_body(status, std::time::Duration::ZERO, body)
                    .await;
            let root = tempfile::tempdir().unwrap();
            let cache = crate::Cache::new(
                root.path().join("cache"),
                root.path().join("vault"),
                root.path().join("tmp"),
            );
            let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
            let result = QueryYoudict { endpoint: url }
                .query_and_store(&query, &cache)
                .await;
            assert_eq!(result.is_err(), is_error);
            if let Ok(entry) = result {
                assert!(entry.not_found());
            }
            assert!(cache.query(&query.cache_key(), "bin").is_err());
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn online_lookup_succeeds_when_cache_is_unwritable() {
        let (url, server) = crate::word::http::tests::server_with_body(
            200, std::time::Duration::ZERO,
            "<div id='phrsListTab'><div class='trans-container'><ul><li>A greeting</li></ul></div></div>",
        ).await;
        let root = tempfile::tempdir().unwrap();
        let blocked = root.path().join("blocked");
        std::fs::write(&blocked, b"not a directory").unwrap();
        let cache = crate::Cache::new(blocked, root.path().join("vault"), root.path().join("tmp"));
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        let entry = QueryYoudict { endpoint: url }
            .query_and_store(&query, &cache)
            .await
            .unwrap();
        assert_eq!(entry.brief, ["A greeting"]);
        server.await.unwrap();
    }

    #[test]
    fn query_parameters_preserve_special_characters() {
        let query = ExactQuery::new("C++ & x=1#tail".into(), Lang::Eng, false).unwrap();
        let url = QueryYoudict::new().url(&query).unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(
            url.query_pairs().collect::<Vec<_>>(),
            [("q".into(), "C++ & x=1#tail".into())]
        );
        assert!(url.fragment().is_none());
    }

    #[test]
    fn unknown_and_verification_pages_are_errors() {
        let query = ExactQuery::new("hello".into(), Lang::Eng, false).unwrap();
        for html in [
            "<html>Please verify your request</html>",
            "<div id='results-contents'><section id='new-layout'>Definition</section></div>",
            "<div id='phrsListTab'></div>",
        ] {
            let error =
                QueryYoudict::parse_document(&Html::parse_document(html), &query).unwrap_err();
            assert!(error.is::<UnexpectedPage>());
        }
    }

    #[test]
    fn recognized_empty_results_are_not_found() {
        let query = ExactQuery::new("unknown".into(), Lang::Eng, false).unwrap();
        let doc = Html::parse_document(
            "<div id='results-contents'><div id='wordArticle' class='trans-wrapper trans-tab'></div><script></script></div>",
        );
        assert!(
            QueryYoudict::parse_document(&doc, &query)
                .unwrap()
                .not_found()
        );
    }

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
