use crate::{Config, WordEntry, WordQuery};
use colored::Colorize;

impl WordEntry {
    pub fn display(&self, word_query: &WordQuery, config: &Config) {
        let normal = &config.normal;

        println!("{}", word_query.word().bright_red());

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
