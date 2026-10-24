use std::collections::HashSet;

use crate::app::config::{Config, Section};
use crate::word::{Answer, PPrint, QueryCache, QueryYoudict, Question};
use crate::{Acquire, Cache, ExactQuery};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

#[derive(Clone, Debug, Serialize, Deserialize, SchemaRead, SchemaWrite)]
pub struct SingleEntry {
    pub pronunciation: Vec<(String, String)>,
    pub brief: Vec<String>,
    pub variants: Vec<String>,
    pub authority: Vec<String>,
    pub sentence: Vec<(String, String)>,
    pub collins: Option<CollinsEntry>,
}

/// The Collins English-Chinese dictionary block, grouped by usage section.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SchemaRead, SchemaWrite)]
pub struct CollinsEntry {
    pub groups: Vec<CollinsGroup>,
}

/// One usage group (for example "VERB USES") of a Collins entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SchemaRead, SchemaWrite)]
pub struct CollinsGroup {
    pub title: Option<String>,
    pub headword: String,
    pub phonetic: Option<String>,
    pub ranks: Option<String>,
    pub word_forms: Option<String>,
    pub senses: Vec<CollinsSense>,
}

/// One numbered Collins sense with its inline Chinese translation and examples.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, SchemaRead, SchemaWrite)]
pub struct CollinsSense {
    pub pos: Option<String>,
    pub definition: String,
    pub translation: String,
    pub examples: Vec<(String, String)>,
}

impl SingleEntry {
    /// Query a word first from cache and then from the web
    pub async fn query(word_query: &ExactQuery, cache: &Cache) -> anyhow::Result<Self> {
        if !word_query.refresh() {
            let cache = cache.clone();
            let query = word_query.clone();
            let cached =
                tokio::task::spawn_blocking(move || QueryCache::new(&cache).acquire(&query))
                    .await
                    .map_err(anyhow::Error::from)
                    .and_then(|result| result);
            match cached {
                Ok(entry) => return Ok(entry),
                Err(err) => {
                    if !err
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|err| err.kind() == std::io::ErrorKind::NotFound)
                    {
                        log::warn!("Ignoring unavailable dictionary cache: {err:#}");
                    }
                }
            }
        }
        QueryYoudict::new().query_and_store(word_query, cache).await
    }

    fn has_youdao(&self) -> bool {
        !self.pronunciation.is_empty()
            || !self.brief.is_empty()
            || !self.variants.is_empty()
            || !self.authority.is_empty()
            || !self.sentence.is_empty()
    }

    fn has_collins(&self) -> bool {
        self.collins
            .as_ref()
            .is_some_and(|collins| !collins.groups.is_empty())
    }

    /// Render the configured sections as colored lines, without printing them.
    fn render(&self, question: &impl Question, config: &Config) -> Vec<String> {
        let sections = config.sections();
        let mut lines = Vec::new();
        let multiple = sections.len() > 1;

        for (index, section) in sections.iter().enumerate() {
            if multiple {
                if index > 0 {
                    lines.push(String::new());
                }
                // A small, dim triangle keeps section labels readable without
                // competing with the definitions themselves.
                lines.push(format!("▾ {}", section.name()).bright_black().to_string());
            }

            match section {
                Section::Youdao if self.has_youdao() => {
                    self.render_youdao(question, config, &mut lines);
                }
                Section::Collins if self.has_collins() => {
                    self.render_collins(config, &mut lines);
                }
                _ => lines.push(
                    format!("({}: no content)", section.name())
                        .bright_black()
                        .to_string(),
                ),
            }
        }

        lines
    }

    fn render_youdao(&self, question: &impl Question, config: &Config, lines: &mut Vec<String>) {
        let youdao = &config.youdao;

        lines.push(question.word().bright_red().to_string());

        if youdao.with_pronunciation && !self.pronunciation.is_empty() {
            let mut line = String::new();
            for (accent, pron) in self.pronunciation.iter() {
                line.push_str(&format!("{} {}\t", accent, pron.cyan()));
            }
            lines.push(line);
        }

        for brief in self.brief.iter() {
            lines.push(brief.bright_blue().to_string());
        }

        if youdao.with_variants {
            for variant in self.variants.iter() {
                lines.push(variant.bright_black().to_string());
            }
        }

        if youdao.with_sentence {
            for (i, (ori, trans)) in self.sentence.iter().enumerate() {
                let idx_str = format!("{}. ", i + 1);
                lines.push(format!("{}{}", idx_str, ori.bright_green()));
                lines.push(format!("{}{}", " ".repeat(idx_str.len()), trans.yellow()));
            }
        }
    }

    fn render_collins(&self, config: &Config, lines: &mut Vec<String>) {
        let Some(collins) = &self.collins else {
            return;
        };
        let options = &config.collins;
        let mut printed_headers = HashSet::new();

        for group in collins.groups.iter() {
            if let Some(title) = &group.title {
                lines.push(format!("[{title}]").bright_black().to_string());
            }

            let mut header_parts = Vec::new();
            if !group.headword.is_empty() {
                header_parts.push(group.headword.bright_red().to_string());
            }
            if let Some(phonetic) = group.phonetic.as_ref().filter(|_| options.with_phonetic) {
                header_parts.push(phonetic.cyan().to_string());
            }
            if let Some(ranks) = &group.ranks {
                header_parts.push(ranks.bright_black().to_string());
            }
            if let Some(word_forms) = group
                .word_forms
                .as_ref()
                .filter(|_| options.with_word_forms)
            {
                header_parts.push(word_forms.bright_black().to_string());
            }
            let header = header_parts.join("  ");
            // Multi-group pages repeat the same heading block for each usage
            // group; only the first occurrence carries new information.
            if !header.is_empty() && printed_headers.insert(header.clone()) {
                lines.push(header);
            }

            for (index, sense) in group.senses.iter().enumerate() {
                let mut line = format!("{}. ", index + 1);
                if let Some(pos) = &sense.pos {
                    line.push_str(&format!("{}  ", pos.cyan()));
                }
                line.push_str(&sense.definition);
                lines.push(line);

                if !sense.translation.is_empty() {
                    lines.push(format!("    {}", sense.translation.yellow()));
                }

                if options.with_examples {
                    for (ori, trans) in sense.examples.iter() {
                        lines.push(format!(
                            "    {}{}",
                            "例：".bright_black(),
                            ori.bright_green()
                        ));
                        if !trans.is_empty() {
                            lines.push(format!("        {}", trans.yellow()));
                        }
                    }
                }
            }
        }
    }

    /// Build the machine-readable representation used by `--format json`.
    ///
    /// All selected sections are listed in order; a selected section without
    /// parsed content carries `null`, matching the human `(section: no content)`
    /// notice.
    pub fn json(&self, question: &impl Question, config: &Config) -> serde_json::Value {
        use serde_json::json;

        let selected = config.sections();
        let sections: Vec<serde_json::Value> = selected
            .iter()
            .map(|section| match section {
                Section::Youdao => {
                    let content = self.has_youdao().then(|| {
                        json!({
                            "pronunciation": &self.pronunciation,
                            "brief": &self.brief,
                            "variants": &self.variants,
                            "sentence": &self.sentence,
                        })
                    });
                    json!({ "section": section.name(), "content": content })
                }
                Section::Collins => {
                    let content = self
                        .collins
                        .as_ref()
                        .filter(|collins| !collins.groups.is_empty());
                    json!({ "section": section.name(), "content": content })
                }
            })
            .collect();

        json!({ "word": question.word(), "sections": sections })
    }

    /// Print the machine-readable representation used by `--format json`.
    pub fn print_json(&self, question: &impl Question, config: &Config) {
        match serde_json::to_string_pretty(&self.json(question, config)) {
            Ok(json) => println!("{json}"),
            Err(err) => log::error!("Failed to serialize query result: {err}"),
        }
    }
}

impl Answer for SingleEntry {
    fn not_found(&self) -> bool {
        !self.has_youdao() && !self.has_collins()
    }
}

impl PPrint for SingleEntry {
    fn pprint(&self, question: &impl Question, config: &Config) {
        for line in self.render(question, config) {
            println!("{line}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::config::Sections;
    use std::path::PathBuf;
    use whatlang::Lang;

    fn collins() -> CollinsEntry {
        CollinsEntry {
            groups: vec![CollinsGroup {
                title: Some("VERB USES".to_owned()),
                headword: "fast".to_owned(),
                phonetic: Some("/fæst/".to_owned()),
                ranks: Some("CET4 TEM4".to_owned()),
                word_forms: Some("(fasting, fasted, fasts)".to_owned()),
                senses: vec![CollinsSense {
                    pos: Some("V-I".to_owned()),
                    definition: "If you fast, you eat nothing for a period of time.".to_owned(),
                    translation: "禁食".to_owned(),
                    examples: vec![(
                        "She fasted for three days.".to_owned(),
                        "她禁食了三天。".to_owned(),
                    )],
                }],
            }],
        }
    }

    fn entry() -> SingleEntry {
        SingleEntry {
            pronunciation: vec![],
            brief: vec!["A greeting".into()],
            variants: vec![],
            authority: vec![],
            sentence: vec![],
            collins: None,
        }
    }

    fn query(word: &str) -> ExactQuery {
        ExactQuery::new(word.to_owned(), Lang::Eng, false).unwrap()
    }

    #[test]
    fn a_word_with_only_collins_content_is_not_missing() {
        let mut entry = entry();
        entry.brief.clear();
        assert!(entry.not_found());
        entry.collins = Some(collins());
        assert!(!entry.not_found());

        entry.collins = Some(CollinsEntry { groups: vec![] });
        assert!(entry.not_found());
    }

    #[test]
    fn multiple_sections_get_headings_and_empty_sections_get_notices() {
        let config = Config::new(PathBuf::new());
        assert_eq!(
            config.sections,
            Sections::All,
            "the default configuration selects every section"
        );

        let lines = entry().render(&query("hello"), &config);
        assert!(lines.iter().any(|line| line.contains("▾ youdao")));
        assert!(lines.iter().any(|line| line.contains("▾ collins")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("(collins: no content)"))
        );
    }

    #[test]
    fn a_single_section_keeps_the_old_clean_output() {
        let mut config = Config::new(PathBuf::new());
        config.sections = Sections::List(vec![Section::Youdao]);
        let lines = entry().render(&query("hello"), &config);
        assert!(!lines.iter().any(|line| line.contains('▾')));
        assert!(lines.iter().any(|line| line.contains("A greeting")));
    }

    #[test]
    fn collins_rendering_includes_groups_senses_and_examples() {
        let mut config = Config::new(PathBuf::new());
        config.sections = Sections::List(vec![Section::Collins]);
        let mut entry = entry();
        entry.collins = Some(collins());

        let lines = entry.render(&query("fast"), &config);
        let text = lines.join("\n");
        assert!(text.contains("[VERB USES]"));
        assert!(text.contains("fast  /fæst/  CET4 TEM4  (fasting, fasted, fasts)"));
        assert!(text.contains("1. V-I  If you fast"));
        assert!(text.contains("禁食"));
        assert!(text.contains("例：She fasted for three days."));
        assert!(text.contains("她禁食了三天。"));
        assert!(!text.contains('▾'), "single sections have no heading");
    }

    #[test]
    fn concise_options_drop_examples_and_word_forms() {
        let mut config = Config::new(PathBuf::new());
        config.sections = Sections::List(vec![Section::Collins]);
        config.collins.with_examples = false;
        config.collins.with_word_forms = false;
        let mut entry = entry();
        entry.collins = Some(collins());

        let text = entry.render(&query("fast"), &config).join("\n");
        assert!(!text.contains("例："));
        assert!(!text.contains("(fasting, fasted, fasts)"));
        assert!(text.contains("禁食"));
    }

    #[test]
    fn json_lists_selected_sections_in_order() {
        let mut config = Config::new(PathBuf::new());
        config.sections = Sections::List(vec![Section::Collins, Section::Youdao]);
        let mut entry = entry();
        entry.sentence = vec![("Hello.".to_owned(), "你好。".to_owned())];

        let json = entry.json(&query("hello"), &config);
        assert_eq!(json["word"], "hello");
        assert_eq!(json["sections"][0]["section"], "collins");
        assert!(json["sections"][0]["content"].is_null());
        assert_eq!(json["sections"][1]["section"], "youdao");
        assert_eq!(json["sections"][1]["content"]["sentence"][0][0], "Hello.");
    }

    #[test]
    fn json_serializes_collins_content() {
        let mut config = Config::new(PathBuf::new());
        config.sections = Sections::List(vec![Section::Collins]);
        let mut entry = entry();
        entry.collins = Some(collins());

        let json = entry.json(&query("fast"), &config);
        assert_eq!(json["sections"][0]["section"], "collins");
        assert_eq!(
            json["sections"][0]["content"]["groups"][0]["title"],
            "VERB USES"
        );
        assert_eq!(
            json["sections"][0]["content"]["groups"][0]["senses"][0]["translation"],
            "禁食"
        );
        assert_eq!(
            json["sections"][0]["content"]["groups"][0]["senses"][0]["examples"][0][0],
            "She fasted for three days."
        );
    }
}
