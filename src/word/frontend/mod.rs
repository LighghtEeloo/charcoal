mod single_entry;

use crate::Question;
use whatlang::Lang;

pub use single_entry::SingleEntry;

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
