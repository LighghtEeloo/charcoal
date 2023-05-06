mod backend;
pub mod frontend;
pub mod speech;

use crate::Config;
use scraper::{ElementRef, Html};
use whatlang::Lang;

pub use backend::*;

pub trait Question {
    fn word(&self) -> String;
    fn inferred_lang(&self) -> Lang;
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
    fn request(self, word_query: &Self::WordQuery) -> anyhow::Result<Html>;
}

trait Select {
    type WordQuery;
    type Target;
    fn select(elem: ElementRef, word_query: &Self::WordQuery) -> anyhow::Result<Self::Target>;
}

pub trait PPrint: Answer {
    fn pprint(&self, question: &impl Question, config: &Config);
}

pub struct ExactQuery {
    word: String,
    lang: Lang,
}

impl<'a> ExactQuery {
    pub fn new(word: String) -> Option<Self> {
        if word.is_empty() {
            return None;
        }
        let lang = whatlang::detect(&word).map_or(Lang::Eng, |info| info.lang());
        Some(Self { word, lang })
    }
}

impl Question for ExactQuery {
    fn word(&self) -> String {
        self.word.clone()
    }
    fn inferred_lang(&self) -> Lang {
        self.lang
    }
}
