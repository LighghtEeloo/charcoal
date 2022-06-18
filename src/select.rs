mod authority;
mod sentence;

use self::sentence::Sentence;
use crate::{entry, WordEntry};
use scraper::{ElementRef, Selector};
use whatlang::Lang;

pub trait Select {
    type Target;
    fn select(elem: ElementRef, lang: &Lang) -> anyhow::Result<Self::Target>;
}

impl Select for entry::FromYoudict {
    type Target = WordEntry;

    fn select(elem: ElementRef, lang: &Lang) -> anyhow::Result<Self::Target> {
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

        let sentence = Sentence::select(elem, lang)?;

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
