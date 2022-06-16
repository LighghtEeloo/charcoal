use scraper::{Html, Selector};

#[derive(Clone, Debug)]
pub struct Word {
    pub word: String,
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<String>,
}

impl Word {
    pub fn is_empty(&self) -> bool {
        return self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty();
    }
}

async fn get_html(url: impl AsRef<str> + reqwest::IntoUrl) -> anyhow::Result<String> {
    let body = reqwest::get(url).await?.text().await?;
    Ok(body)
}

fn trim_str(t: &str) -> Option<String> {
    let t = t.trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

impl Word {
    pub async fn query(query_word: impl ToString) -> anyhow::Result<Word> {
        let word = query_word.to_string();
        let youdao_dict_url =
            url::Url::parse(&format!("http://dict.youdao.com/search?q={}", word))?;

        let xml = get_html(youdao_dict_url).await?;
        let doc = Html::parse_document(&xml);

        let pronunciation = {
            let mut vec = Vec::new();
            let sel = Selector::parse("span.pronounce").unwrap();
            for child in doc.select(&sel) {
                let pron = child.text().filter_map(trim_str).collect::<Vec<String>>();
                vec.push((pron[0].to_owned(), pron[1].to_owned()))
            }
            vec
        };
        let brief = {
            let mut vec = Vec::new();
            let sel = Selector::parse("#phrsListTab .trans-container ul li").unwrap();
            for child in doc.select(&sel) {
                vec.push(
                    child
                        .text()
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join(""),
                );
            }
            vec
        };
        let variants = {
            let mut vec = Vec::new();
            let sel = Selector::parse("#phrsListTab .trans-container p").unwrap();
            for child in doc.select(&sel) {
                vec.extend(child.text().map(|t| {
                    t.split("\n")
                        .filter_map(trim_str)
                        .collect::<Vec<String>>()
                        .join(" ")
                }));
            }
            vec
        };

        Ok(Word {
            word,
            pronunciation,
            brief,
            variants,
            authority: Vec::new(),
            sentence: Vec::new(),
        })
    }
}
