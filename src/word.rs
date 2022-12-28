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
    fn pprint(&self, word_query: &impl Question, config: &Config);
}
