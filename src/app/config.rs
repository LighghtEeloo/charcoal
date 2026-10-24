use super::cli::{QueryArgs, SectionArg, Toggle};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use whatlang::Lang;

/// A displayable content section, in the order configured by the user.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    /// Youdao's own basic translation block.
    #[serde(alias = "basic")]
    Youdao,
    /// The Collins English-Chinese dictionary block.
    Collins,
}

impl Section {
    pub fn name(self) -> &'static str {
        match self {
            Self::Youdao => "youdao",
            Self::Collins => "collins",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Youdao, Self::Collins]
    }
}

/// Selection of content sections to display.
///
/// The configuration accepts either `sections = true` (enable every section in
/// the canonical order) or an explicit list of section names in rendering
/// order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Sections {
    /// Enable every known section in the canonical order.
    #[default]
    All,
    /// Enable the listed sections in the listed order.
    List(Vec<Section>),
}

impl Sections {
    pub fn resolve(&self) -> Vec<Section> {
        match self {
            Self::All => Section::all(),
            Self::List(sections) => sections.clone(),
        }
    }
}

impl Serialize for Sections {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::All => serializer.serialize_bool(true),
            Self::List(sections) => sections.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Sections {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Flag(bool),
            List(Vec<Section>),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Flag(true) => Ok(Self::All),
            Repr::Flag(false) => Err(serde::de::Error::custom(
                "`sections = false` is not supported; use a list of sections or `sections = true` for all",
            )),
            Repr::List(sections) => Ok(Self::List(sections)),
        }
    }
}

impl From<SectionArg> for Section {
    fn from(section: SectionArg) -> Self {
        match section {
            SectionArg::Youdao => Self::Youdao,
            SectionArg::Collins => Self::Collins,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    #[serde(skip)]
    pub path: PathBuf,
    /// Content sections to display, in rendering order. `true` enables every
    /// section in its canonical order.
    pub sections: Sections,
    pub lang: Lang,
    pub speak: bool,
    pub youdao: Youdao,
    pub collins: Collins,
}

/// Detail switches for Youdao's basic translation block.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Youdao {
    pub with_pronunciation: bool,
    pub with_variants: bool,
    pub with_sentence: bool,
}

/// Detail switches for the Collins dictionary block.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Collins {
    pub with_phonetic: bool,
    pub with_word_forms: bool,
    pub with_examples: bool,
}

/// The pre-`sections` configuration format, accepted so old files can be
/// upgraded in place.
#[derive(Deserialize)]
struct LegacyConfig {
    #[serde(default)]
    main_mode: Option<LegacyMainMode>,
    #[serde(default)]
    lang: Option<Lang>,
    #[serde(default)]
    speak: Option<bool>,
    #[serde(default, rename = "Normal")]
    youdao: Option<Youdao>,
    #[serde(default, rename = "Collins")]
    collins: Option<Collins>,
}

#[derive(Deserialize)]
enum LegacyMainMode {
    Normal,
    Collins,
    Both,
}

impl LegacyConfig {
    fn into_config(self, path: PathBuf) -> Config {
        Config {
            path,
            sections: match self.main_mode {
                Some(LegacyMainMode::Collins) => Sections::List(vec![Section::Collins]),
                Some(LegacyMainMode::Both) => Sections::All,
                // The old enum only exercised `Normal`, which only displayed
                // Youdao content; a missing mode means the same thing.
                Some(LegacyMainMode::Normal) | None => Sections::List(vec![Section::Youdao]),
            },
            lang: self.lang.unwrap_or(Lang::Eng),
            speak: self.speak.unwrap_or(false),
            youdao: self.youdao.unwrap_or_default(),
            collins: self.collins.unwrap_or_default(),
        }
    }
}

impl Config {
    pub fn new(path: PathBuf) -> Self {
        Config {
            path,
            sections: Sections::All,
            lang: Lang::Eng,
            speak: false,
            youdao: Youdao::default(),
            collins: Collins::default(),
        }
    }

    pub fn of_file(path: PathBuf) -> anyhow::Result<Self> {
        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read configuration at {}", path.display()))?;
        Self::from_toml(&content, path.clone())
            .with_context(|| format!("Invalid configuration at {}", path.display()))
    }

    fn from_toml(content: &str, path: PathBuf) -> anyhow::Result<Self> {
        let value: toml::Value = toml::from_str(content)?;
        let legacy = value.as_table().is_some_and(|table| {
            table.contains_key("main_mode")
                || table.contains_key("Normal")
                || table.contains_key("Collins")
        });

        if legacy {
            let legacy: LegacyConfig = toml::from_str(content)?;
            let config = legacy.into_config(path.clone());
            // A failed rewrite must not make a query fail: the migrated
            // in-memory configuration is still complete and usable.
            if let Err(err) = config.to_file() {
                log::warn!(
                    "Failed to upgrade configuration at {}: {err:#}",
                    path.display()
                );
            }
            Ok(config)
        } else {
            let mut config: Self = toml::from_str(content)?;
            config.path = path;
            config.normalize();
            Ok(config)
        }
    }

    /// The sections to render, in order.
    pub fn sections(&self) -> Vec<Section> {
        self.sections.resolve()
    }

    /// Drop duplicate sections while keeping the first occurrence; an empty
    /// list falls back to enabling all sections.
    fn normalize(&mut self) {
        let normalized = if let Sections::List(sections) = &self.sections {
            let mut unique: Vec<Section> = Vec::new();
            for section in sections.iter().copied() {
                if !unique.contains(&section) {
                    unique.push(section);
                }
            }
            Some(if unique.is_empty() {
                Sections::All
            } else {
                Sections::List(unique)
            })
        } else {
            None
        };
        if let Some(normalized) = normalized {
            self.sections = normalized;
        }
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
        pending.write_all(toml::to_string(self)?.as_bytes())?;
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

        if args.all {
            self.sections = Sections::All;
        } else if let Some(sections) = &args.sections {
            self.sections = Sections::List(
                sections
                    .iter()
                    .copied()
                    .map(Section::from)
                    .collect::<Vec<_>>(),
            );
            self.normalize();
        }

        if args.concise {
            args.concise_as = Some(Toggle::True);
        }
        if let Some(concise_as) = &args.concise_as {
            concise_as.counter_twitch(&mut self.youdao.with_sentence);
            concise_as.counter_twitch(&mut self.youdao.with_variants);
            concise_as.counter_twitch(&mut self.collins.with_examples);
            concise_as.counter_twitch(&mut self.collins.with_word_forms);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new(PathBuf::new())
    }
}

impl Default for Youdao {
    fn default() -> Self {
        Self {
            with_pronunciation: true,
            with_variants: true,
            with_sentence: true,
        }
    }
}

impl Default for Collins {
    fn default() -> Self {
        Self {
            with_phonetic: true,
            with_word_forms: true,
            with_examples: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::cli::{Cli, Commands};
    use clap::Parser;

    fn query_args(argv: &[&str]) -> QueryArgs {
        match Cli::try_parse_from(argv).unwrap().commands {
            Commands::Query(args) => args,
            _ => panic!("expected a query command"),
        }
    }

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
    fn missing_configuration_is_initialized_with_all_sections_by_default() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.path, path);
        assert_eq!(config.sections, Sections::All);
        assert_eq!(config.sections(), [Section::Youdao, Section::Collins]);
        assert!(config.youdao.with_sentence);
        assert!(config.collins.with_examples);

        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("sections = true"), "{written}");
        assert!(written.contains("[youdao]"), "{written}");
        assert!(written.contains("[collins]"), "{written}");
        assert!(!Config::of_file(path).unwrap().speak);
    }

    #[test]
    fn missing_fields_use_defaults_without_rewriting_the_file() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "speak = true\n\n[youdao]\nwith_sentence = false\n";
        fs::write(&path, original).unwrap();
        let config = Config::load_or_create(path.clone()).unwrap();
        assert!(config.speak);
        assert_eq!(config.lang, Lang::Eng);
        assert!(!config.youdao.with_sentence);
        assert!(config.youdao.with_variants);
        assert_eq!(config.sections, Sections::All);
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn legacy_configuration_is_upgraded_in_place() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "main_mode = \"Normal\"\nlang = \"eng\"\nspeak = true\n\n[Normal]\nwith_pronunciation = true\nwith_variants = false\nwith_sentence = true\n";
        fs::write(&path, original).unwrap();

        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.sections, Sections::List(vec![Section::Youdao]));
        assert!(config.speak);
        assert!(config.youdao.with_pronunciation);
        assert!(!config.youdao.with_variants);
        assert!(config.youdao.with_sentence);

        let upgraded = fs::read_to_string(&path).unwrap();
        assert!(upgraded.contains("sections = [\"youdao\"]"), "{upgraded}");
        assert!(upgraded.contains("[youdao]"), "{upgraded}");
        assert!(!upgraded.contains("main_mode"), "{upgraded}");
        assert!(!upgraded.contains("[Normal]"), "{upgraded}");

        // The upgraded file round-trips without another migration.
        let again = Config::of_file(path).unwrap();
        assert_eq!(again.sections, Sections::List(vec![Section::Youdao]));
        assert!(!again.youdao.with_variants);
    }

    #[test]
    fn legacy_both_mode_upgrades_to_both_sections() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(
            &path,
            "main_mode = \"Both\"\n[Collins]\nwith_examples = false\n",
        )
        .unwrap();
        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.sections, Sections::All);
        assert!(!config.collins.with_examples);
        assert!(
            fs::read_to_string(path)
                .unwrap()
                .contains("sections = true")
        );
    }

    #[test]
    fn legacy_configuration_without_mode_only_shows_youdao() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(&path, "[Normal]\nwith_sentence = false\n").unwrap();
        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.sections, Sections::List(vec![Section::Youdao]));
        assert!(!config.youdao.with_sentence);
    }

    #[cfg(unix)]
    #[test]
    fn upgrade_rewrite_failure_does_not_block_loading() {
        use std::os::unix::fs::PermissionsExt;

        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(&path, "main_mode = \"Normal\"\n").unwrap();

        let mut permissions = fs::metadata(root.path()).unwrap().permissions();
        permissions.set_mode(0o500);
        fs::set_permissions(root.path(), permissions).unwrap();

        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.sections, Sections::List(vec![Section::Youdao]));

        let mut permissions = fs::metadata(root.path()).unwrap().permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(root.path(), permissions).unwrap();
    }

    #[test]
    fn explicit_reset_replaces_malformed_configuration() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(&path, "invalid = [").unwrap();
        Config::new(path.clone()).to_file().unwrap();
        let config = Config::of_file(path).unwrap();
        assert!(!config.speak);
        assert_eq!(config.sections, Sections::All);
    }

    #[test]
    fn sections_true_enables_every_section() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "sections = true\n";
        fs::write(&path, original).unwrap();
        let config = Config::load_or_create(path.clone()).unwrap();
        assert_eq!(config.sections, Sections::All);
        assert_eq!(config.sections(), [Section::Youdao, Section::Collins]);
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn sections_false_is_rejected_without_rewriting() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "sections = false\n";
        fs::write(&path, original).unwrap();
        let error = Config::load_or_create(path.clone()).err().unwrap();
        assert!(error.to_string().contains(path.to_str().unwrap()));
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn unknown_sections_are_rejected_without_rewriting() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let original = "sections = [\"nope\"]\n";
        fs::write(&path, original).unwrap();
        assert!(Config::load_or_create(path.clone()).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), original);
    }

    #[test]
    fn duplicate_sections_are_deduplicated() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        fs::write(&path, "sections = [\"youdao\", \"collins\", \"youdao\"]\n").unwrap();
        let config = Config::load_or_create(path).unwrap();
        assert_eq!(
            config.sections,
            Sections::List(vec![Section::Youdao, Section::Collins])
        );
    }

    #[test]
    fn query_flags_override_sections_and_apply_the_concise_preset() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("config.toml");
        let mut config = Config::new(path);

        let mut args = query_args(&[
            "charcoal",
            "query",
            "hello",
            "--sections",
            "collins,youdao,collins",
            "-c",
        ]);
        config.apply(&mut args);
        assert_eq!(
            config.sections,
            Sections::List(vec![Section::Collins, Section::Youdao])
        );
        assert!(!config.youdao.with_sentence);
        assert!(!config.youdao.with_variants);
        assert!(!config.collins.with_examples);
        assert!(!config.collins.with_word_forms);
        // Pronunciation switches are not part of the concise preset.
        assert!(config.youdao.with_pronunciation);
        assert!(config.collins.with_phonetic);

        let mut args = query_args(&["charcoal", "query", "hello", "--sections", "youdao"]);
        config.apply(&mut args);
        assert_eq!(config.sections, Sections::List(vec![Section::Youdao]));

        // `-a` overrides a narrowed configuration for this query.
        let mut args = query_args(&["charcoal", "query", "hello", "-a"]);
        config.apply(&mut args);
        assert_eq!(config.sections, Sections::All);
    }

    #[test]
    fn all_flag_conflicts_with_explicit_sections() {
        assert!(
            Cli::try_parse_from([
                "charcoal",
                "query",
                "hello",
                "--all",
                "--sections",
                "youdao",
            ])
            .is_err()
        );
        assert!(Cli::try_parse_from(["charcoal", "query", "hello", "-a"]).is_ok());
    }
}
