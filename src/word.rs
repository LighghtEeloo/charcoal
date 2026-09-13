mod backend;
pub mod frontend;
mod http;
pub mod speech;

use crate::Config;
use scraper::{ElementRef, Html};
use whatlang::Lang;

pub use backend::*;

pub trait Question {
    fn word(&self) -> String;
    fn lang(&self) -> Lang;
    fn refresh(&self) -> bool {
        false
    }
}
pub trait Answer {
    fn not_found(&self) -> bool;
}

pub trait Acquire {
    type WordQuery;
    type WordEntry;
    fn acquire(self, word_query: &Self::WordQuery) -> anyhow::Result<Self::WordEntry>;
}

trait Request {
    type WordQuery;
    async fn request(
        self, word_query: &Self::WordQuery, client: &reqwest::Client,
    ) -> anyhow::Result<Html>;
}

trait Select {
    type WordQuery;
    type Target;
    fn select(elem: ElementRef, word_query: &Self::WordQuery) -> anyhow::Result<Self::Target>;
}

pub trait PPrint: Answer {
    fn pprint(&self, question: &impl Question, config: &Config);
}

#[derive(Clone)]
pub struct ExactQuery {
    word: String,
    lang: Lang,
    refresh: bool,
}

impl<'a> ExactQuery {
    pub fn new(word: String, lang: Lang, refresh: bool) -> Option<Self> {
        if word.is_empty() {
            return None;
        }
        // let lang = whatlang::detect(&word).map_or(Lang::Eng, |info| info.lang());
        Some(Self {
            word,
            lang,
            refresh,
        })
    }
}

impl Question for ExactQuery {
    fn word(&self) -> String {
        self.word.clone()
    }
    fn lang(&self) -> Lang {
        self.lang
    }
    fn refresh(&self) -> bool {
        self.refresh
    }
}
