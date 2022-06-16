use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io, path::Path};

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

#[derive(Serialize, Deserialize)]
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
    pub fn of_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
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

pub struct ConfigBuilder {
    qualifier: &'static str,
    organization: &'static str,
    application: &'static str,
    config_file: &'static str,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            qualifier: "",
            organization: "LitiaEeloo",
            application: "Charcoal",
            config_file: "charcoal.toml",
        }
    }
}

impl ConfigBuilder {
    pub fn build(self) -> anyhow::Result<Config> {
        let project_dirs = directories_next::ProjectDirs::from(
            self.qualifier,
            self.organization,
            self.application,
        )
        .expect("No valid config directory fomulated");
        let mut config_path = project_dirs.config_dir().to_path_buf();
        fs::create_dir_all(&config_path)?;
        config_path.push(self.config_file);

        Config::of_path(&config_path).map_or_else(
            |_err| -> anyhow::Result<Config> {
                println!(
                    "Creating new configuration file at: \n\t{}",
                    config_path.display()
                );
                let config = Config::default();
                config.to_file(&config_path)?;
                Ok(config)
            },
            |config| Ok(config),
        )
    }
}
