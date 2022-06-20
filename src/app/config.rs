use super::cli::{QueryArgs, Toggle};
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,
    pub main_mode: MainMode,
    pub speak: bool,
    #[serde(rename = "Normal")]
    pub normal: Normal,
}

#[derive(Serialize, Deserialize)]
pub enum MainMode {
    Normal,
    // Collins,
    // Both,
}

#[derive(Serialize, Deserialize)]
pub struct Normal {
    pub with_pronunciation: bool,
    pub with_variants: bool,
    pub with_sentence: bool,
}

impl Config {
    pub fn new(path: PathBuf) -> Self {
        Config {
            path,
            main_mode: MainMode::Normal,
            normal: Normal {
                with_pronunciation: true,
                with_variants: true,
                with_sentence: true,
            },
            speak: false,
        }
    }
    pub fn of_file(path: PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(&path)?;
        let config = toml::from_str(&content)?;
        Ok(Self { path, ..config })
    }
    pub fn to_file(&self) -> anyhow::Result<()> {
        let s = toml::to_string_pretty(&self)?;
        fs::write(&self.path, s)?;
        Ok(())
    }
    pub fn apply(&mut self, args: &mut QueryArgs) {
        if args.speak {
            args.speak_as = Some(Toggle::True);
        }
        if args.mute {
            args.speak_as = Some(Toggle::False);
        }
        if let Some(speak_as) = &args.speak_as {
            speak_as.twitch(&mut self.speak);
        }

        if args.concise {
            args.concise_as = Some(Toggle::True);
        }
        if let Some(concise_as) = &args.concise_as {
            concise_as.counter_twitch(&mut self.normal.with_sentence);
            concise_as.counter_twitch(&mut self.normal.with_variants);
        }
    }
}
