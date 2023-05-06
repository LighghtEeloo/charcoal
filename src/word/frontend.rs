use crate::word::{Answer, Config, PPrint, QueryCache, QueryYoudict, Question};
use crate::{Acquire, Cache, ExactQuery};
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SingleEntry {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
}

impl SingleEntry {
    /// Query a word first from cache and then from the web
    pub async fn query(word_query: &ExactQuery, cache: &Cache) -> anyhow::Result<Self> {
        (QueryCache::new(cache).acquire(word_query))
            .or_else(|_err| QueryYoudict::new().query_and_store(word_query, cache))
    }
}

impl Answer for SingleEntry {
    fn not_found(&self) -> bool {
        self.pronunciation.is_empty()
            && self.brief.is_empty()
            && self.variants.is_empty()
            && self.authority.is_empty()
            && self.sentence.is_empty()
    }
}

impl PPrint for SingleEntry {
    fn pprint(&self, question: &impl Question, config: &Config) {
        let normal = &config.normal;

        println!("{}", question.word().bright_red());

        if normal.with_pronunciation && !self.pronunciation.is_empty() {
            for (accent, pron) in self.pronunciation.iter() {
                print!("{} {}\t", accent, pron.cyan())
            }
            println!();
        }

        for line in self.brief.iter() {
            println!("{}", line.bright_blue())
        }

        if normal.with_variants {
            for line in self.variants.iter() {
                println!("{}", line.bright_black())
            }
        }

        if normal.with_sentence {
            for (i, (ori, trans)) in self.sentence.iter().enumerate() {
                let idx_str = format!("{}. ", i + 1);
                let idx_blank = " ".repeat(idx_str.len());
                print!(
                    "{}{}\n{}{}\n",
                    idx_str,
                    ori.bright_green(),
                    idx_blank,
                    trans.yellow()
                )
            }
        }
    }
}
