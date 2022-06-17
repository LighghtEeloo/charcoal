use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
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
        [
            WithPronunciation,
            WithVariants,
            WithAuthority,
            WithSentence,
            WithSpeech,
        ]
        .into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    pub toggles: HashSet<Toggle>,
}

impl Config {
    pub fn all() -> Self {
        Self {
            path: None,
            toggles: Toggle::all().collect(),
        }
    }
    pub fn of_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = fs::read_to_string(&path)?;
        Ok(Self {
            path: Some(path.as_ref().to_path_buf()),
            ..toml::from_str(&content)?
        })
    }
    pub fn to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let s = toml::to_string_pretty(&self)?;
        fs::write(path, s)?;
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        use Toggle::*;
        let mut config = Config::all();
        config.turn_off(WithSpeech);
        config
    }
}

impl Config {
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
