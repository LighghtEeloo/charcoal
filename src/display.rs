use crate::{Config, WordEntry};
use colored::Colorize;

impl WordEntry {
    pub fn display(&self, word: impl AsRef<str>, config: &Config) {
        print!("{}\n", word.as_ref().bright_red());

        if config.normal.with_pronunciation {
            for (accent, pron) in self.pronunciation.iter() {
                print!("{} {}\t", accent, pron.cyan())
            }
            print!("\n");
        }

        for line in self.brief.iter() {
            print!("{}\n", line.bright_blue())
        }

        if config.normal.with_variants {
            for line in self.variants.iter() {
                print!("{}\n", line.bright_black())
            }
        }

        if config.normal.with_sentence {
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
