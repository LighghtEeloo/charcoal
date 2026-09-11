use super::cli::{QueryArgs, Toggle};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use whatlang::Lang;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,
    pub main_mode: MainMode,
    pub lang: Lang,
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
#[serde(default)]
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
            lang: Lang::Eng,
            speak: false,
            normal: Normal {
                with_pronunciation: true,
                with_variants: true,
                with_sentence: true,
            },
        }
    }
    pub fn of_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read configuration at {}", path.display()))?;
        let config = toml::from_str(&content)
            .with_context(|| format!("Invalid configuration at {}", path.display()))?;
        Ok(Self { path, ..config })
    }

    pub fn load_or_create(path: PathBuf) -> anyhow::Result<Self> {
        match Self::of_file(path.clone()) {
            Ok(config) => Ok(config),
            Err(err)
                if err
                    .downcast_ref::<io::Error>()
                    .is_some_and(|err| err.kind() == io::ErrorKind::NotFound) =>
            {
                let config = Self::new(path.clone());
                let pending = config.pending_file()?;
                match pending.persist_noclobber(&path) {
                    Ok(_) => Ok(config),
                    // Another process may have initialized or edited the file meanwhile.
                    Err(err) if err.error.kind() == io::ErrorKind::AlreadyExists => {
                        Self::of_file(path)
                    }
                    Err(err) => Err(err.error).context("Failed to initialize configuration"),
                }
            }
            Err(err) => Err(err),
        }
    }

    fn pending_file(&self) -> anyhow::Result<tempfile::NamedTempFile> {
        let parent = self
            .path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."));
        let mut pending = tempfile::NamedTempFile::new_in(parent)?;
        pending.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        pending.as_file().sync_all()?;
        Ok(pending)
    }

    pub fn to_file(&self) -> anyhow::Result<()> {
        self.pending_file()?
            .persist(&self.path)
            .map_err(|err| err.error)
            .with_context(|| format!("Failed to write configuration at {}", self.path.display()))?;
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

impl Default for Config {
    fn default() -> Self {
        Self::new(PathBuf::new())
    }
}

impl Default for Normal {
    fn default() -> Self {
        Self {
            with_pronunciation: true,
            with_variants: true,
            with_sentence: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_configuration_is_preserved() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "speak = true\n[Normal\n";
        fs::write(&path, original).unwrap();
        let error = Config::load_or_create(path.clone()).err().unwrap();
        assert!(error.to_string().contains(path.to_str().unwrap()));
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn unreadable_configuration_is_not_replaced() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::create_dir(&path).unwrap();
        assert!(Config::load_or_create(path.clone()).is_err());
        assert!(path.is_dir());
    }

    #[test]
    fn missing_configuration_is_initialized() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.path, path);
        assert!(!Config::of_file(path).unwrap().speak);
    }

    #[test]
    fn missing_fields_use_defaults_without_rewriting_the_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "speak = true\n[Normal]\nwith_sentence = false\n";
        fs::write(&path, original).unwrap();
        let config = Config::load_or_create(path.clone()).unwrap();
        assert!(config.speak);
        assert_eq!(config.lang, Lang::Eng);
        assert!(!config.normal.with_sentence);
        assert!(config.normal.with_variants);
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn explicit_reset_replaces_malformed_configuration() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(&path, "invalid = [").unwrap();
        Config::new(path.clone()).to_file().unwrap();
        assert!(!Config::of_file(path).unwrap().speak);
    }
}
