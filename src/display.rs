use colored::Colorize;
use std::fmt::Display;

use super::Word;

// pub enum Sector {
//     Face,
//     Pronunciation,
//     Brief,
//     Variants,
//     Authority,
//     Sentence
// }

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n", self.word.bright_red())?;

        for (accent, pron) in self.pronunciation.iter() {
            write!(f, "{} {}\t", accent, pron.cyan())?
        }
        write!(f, "\n")?;

        for line in self.brief.iter() {
            write!(f, "{}\n", line.bright_blue())?
        }

        for line in self.variants.iter() {
            write!(f, "{}\n", line.bright_black())?
        }

        Ok(())
    }
}
