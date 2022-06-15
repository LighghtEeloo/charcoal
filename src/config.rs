use std::collections::HashSet;

pub struct Config {
    pub toggles: HashSet<Toggle>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            toggles: HashSet::new(),
        }
    }
    pub fn all() -> Self {
        Self {
            toggles: Toggle::all().collect(),
        }
    }
    pub fn check(&self, toggle: Toggle) -> bool {
        self.toggles.contains(&toggle)
    }
    pub fn turn_on(&mut self, toggle: Toggle) {
        self.toggles.insert(toggle);
    }
    pub fn turn_off(&mut self, toggle: Toggle) {
        self.toggles.remove(&toggle);
    }
    pub fn flip(&mut self, toggle: Toggle) {
        if self.check(toggle) {
            self.turn_off(toggle)
        } else {
            self.turn_on(toggle)
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Toggle {
    WithPronunciation,
    WithVariants,
    WithAuthority,
    WithSentence,
    WithSpeech,
}

impl Toggle {
    pub fn all() -> impl Iterator<Item = Toggle> {
        use Toggle::*;
        vec![
            WithPronunciation,
            WithVariants,
            WithAuthority,
            WithSentence,
            WithSpeech,
        ]
        .into_iter()
    }
}
