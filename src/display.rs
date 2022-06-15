use colored::Colorize;

use super::{Config, Toggle, Word};

impl Word {
    pub fn display(&self, config: &Config) {
        use Toggle::*;
        print!("{}\n", self.word.bright_red());

        if config.check(WithPronunciation) {
            for (accent, pron) in self.pronunciation.iter() {
                print!("{} {}\t", accent, pron.cyan())
            }
            print!("\n");
        }

        for line in self.brief.iter() {
            print!("{}\n", line.bright_blue())
        }

        if config.check(WithVariants) {
            for line in self.variants.iter() {
                print!("{}\n", line.bright_black())
            }
        }
    }
}
