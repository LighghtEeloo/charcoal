use crate::{Config, WordEntry};
use colored::Colorize;
use whatlang::Lang;

impl WordEntry {
    pub fn display(&self, word: impl AsRef<str>, _lang: &Lang, config: &Config) {
        let normal = &config.normal;

        print!("{}\n", word.as_ref().bright_red());

        if normal.with_pronunciation && !self.pronunciation.is_empty() {
            for (accent, pron) in self.pronunciation.iter() {
                print!("{} {}\t", accent, pron.cyan())
            }
            print!("\n");
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
