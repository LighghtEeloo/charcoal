use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WordEntry {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
}

impl WordEntry {
    pub fn is_empty(&self) -> bool {
        return self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty();
    }
}