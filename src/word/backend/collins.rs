use crate::word::frontend::{CollinsEntry, CollinsGroup, CollinsSense};
use scraper::{ElementRef, Selector};

use super::youdict::{normalize_whitespace, paragraph_text};

/// Parse the Collins English-Chinese dictionary block from a Youdao result page.
///
/// The block is optional: Chinese queries and words outside Collins coverage do
/// not contain it, so callers must treat `None` as "no Collins content".
pub(super) fn parse(root: ElementRef<'_>) -> Option<CollinsEntry> {
    let container_sel = Selector::parse("#collinsResult").ok()?;
    let container = root.select(&container_sel).next()?;

    let toggle_sel = Selector::parse(".collinsToggle").unwrap();
    let toggle = container.select(&toggle_sel).next()?;

    let group_sel = Selector::parse(".wt-container").unwrap();
    let title_sel = Selector::parse(".title.trans-tip").unwrap();
    let heading_sel = Selector::parse("h4").unwrap();
    let headword_sel = Selector::parse("span.title").unwrap();
    let phonetic_sel = Selector::parse("em.phonetic").unwrap();
    let rank_sel = Selector::parse("span.rank").unwrap();
    let pattern_sel = Selector::parse("span.additional.pattern").unwrap();
    let item_sel = Selector::parse("li").unwrap();
    let major_sel = Selector::parse(".collinsMajorTrans").unwrap();
    let paragraph_sel = Selector::parse("p").unwrap();
    let example_lists_sel = Selector::parse(".exampleLists").unwrap();
    let examples_sel = Selector::parse(".examples").unwrap();

    let mut groups: Vec<CollinsGroup> = Vec::new();
    for group in toggle.select(&group_sel) {
        let title = group.select(&title_sel).next().map(paragraph_text);
        let heading = group.select(&heading_sel).next();
        let headword = heading
            .and_then(|heading| heading.select(&headword_sel).next())
            .map(paragraph_text)
            .unwrap_or_default();
        let phonetic = heading
            .and_then(|heading| heading.select(&phonetic_sel).next())
            .map(paragraph_text);
        let ranks = heading
            .and_then(|heading| heading.select(&rank_sel).next())
            .map(paragraph_text);
        let word_forms = heading
            .and_then(|heading| heading.select(&pattern_sel).next())
            .map(paragraph_text)
            .map(tidy_word_forms);

        let mut senses = Vec::new();
        for item in group.select(&item_sel) {
            // Multi-group entries keep part-of-speech subheadings in bare `li`
            // elements; they are navigation, not dictionary senses.
            let Some(major) = item.select(&major_sel).next() else {
                continue;
            };
            if let Some(sense) = parse_sense(
                &major,
                &item,
                &paragraph_sel,
                &example_lists_sel,
                &examples_sel,
            ) {
                senses.push(sense);
            }
        }

        if senses.is_empty() {
            continue;
        }
        groups.push(CollinsGroup {
            title,
            headword,
            phonetic,
            ranks,
            word_forms,
            senses,
        });
    }

    if groups.is_empty() {
        None
    } else {
        Some(CollinsEntry { groups })
    }
}

fn parse_sense(
    major: &ElementRef<'_>, item: &ElementRef<'_>, paragraph_sel: &Selector,
    example_lists_sel: &Selector, examples_sel: &Selector,
) -> Option<CollinsSense> {
    let paragraph = major.select(paragraph_sel).next()?;

    let mut additional = Vec::new();
    for child in paragraph.child_elements() {
        if child.value().name() == "span"
            && child.value().classes().any(|class| class == "additional")
        {
            additional.push(paragraph_text(child));
        }
    }
    let mut additional = additional.into_iter();
    let pos = additional.next();
    let notes: Vec<String> = additional.collect();

    let mut raw = String::new();
    collect_text(paragraph, &mut raw);
    let raw = normalize_whitespace(&raw);
    let (definition, mut translation) = split_translation(&raw);
    if !notes.is_empty() {
        let notes = notes.join(" ");
        translation = if translation.is_empty() {
            notes
        } else {
            format!("{translation} {notes}")
        };
    }

    let mut examples = Vec::new();
    for list in item.select(example_lists_sel) {
        for pair in list.select(examples_sel) {
            let mut paragraphs = pair
                .child_elements()
                .filter(|element| element.value().name() == "p");
            let (Some(ori), Some(trans)) = (paragraphs.next(), paragraphs.next()) else {
                continue;
            };
            let ori = paragraph_text(ori);
            let trans = paragraph_text(trans);
            if ori.is_empty() && trans.is_empty() {
                continue;
            }
            examples.push((ori, trans));
        }
    }

    if definition.is_empty() && translation.is_empty() && examples.is_empty() {
        return None;
    }
    Some(CollinsSense {
        pos,
        definition,
        translation,
        examples,
    })
}

/// Gather text while dropping the structural `span.additional` children (POS
/// markers and register notes) that are parsed separately.
fn collect_text(node: ElementRef<'_>, out: &mut String) {
    for child in node.children() {
        match child.value() {
            scraper::Node::Text(text) => out.push_str(&text.text),
            scraper::Node::Element(_) => {
                let Some(element) = ElementRef::wrap(child) else {
                    continue;
                };
                if element.value().name() == "span"
                    && element.value().classes().any(|class| class == "additional")
                {
                    continue;
                }
                collect_text(element, out);
            }
            _ => {}
        }
    }
}

/// Split a Collins sense paragraph into its English definition and the inlined
/// Chinese translation. The translation starts at the first CJK character; the
/// split point backs over whitespace and opening parentheses so that characters
/// like `(常指不情愿地) 承认` stay on the translation side.
fn split_translation(text: &str) -> (String, String) {
    let Some(start) = text
        .char_indices()
        .find(|(_, character)| is_cjk(*character))
        .map(|(index, _)| index)
    else {
        return (text.trim().to_owned(), String::new());
    };

    let mut split = start;
    while split > 0 {
        let previous = text[..split].chars().next_back().unwrap();
        if previous.is_whitespace() || matches!(previous, '(' | '（') {
            split -= previous.len_utf8();
        } else {
            break;
        }
    }
    (
        text[..split].trim_end().to_owned(),
        text[split..].trim_start().to_owned(),
    )
}

fn is_cjk(character: char) -> bool {
    matches!(character as u32, 0x3400..=0x4dbf | 0x4e00..=0x9fff | 0xf900..=0xfaff)
}

fn tidy_word_forms(forms: String) -> String {
    forms.replace("( ", "(").replace(" )", ")")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_html(html: &str) -> Option<CollinsEntry> {
        let doc = scraper::Html::parse_document(html);
        parse(doc.root_element())
    }

    #[test]
    fn parses_flat_collins_entries() {
        let entry = parse_html(include_str!("fixtures/collins_flat.html")).unwrap();
        assert_eq!(entry.groups.len(), 1);

        let group = &entry.groups[0];
        assert_eq!(group.title, None);
        assert_eq!(group.headword, "concede");
        assert_eq!(group.phonetic.as_deref(), Some("/kənˈsiːd/"));
        assert_eq!(group.ranks.as_deref(), Some("CET4 TEM8"));
        assert_eq!(
            group.word_forms.as_deref(),
            Some("(conceding, conceded, concedes)")
        );
        assert_eq!(group.senses.len(), 5);

        let sense = &group.senses[0];
        assert_eq!(sense.pos.as_deref(), Some("V-T"));
        assert_eq!(
            sense.definition,
            "If you concede something, you admit, often unwillingly, that it is true or correct."
        );
        assert_eq!(sense.translation, "(常指不情愿地) 承认");
        assert_eq!(
            sense.examples[0],
            (
                "Bess finally conceded that Nancy was right.".to_owned(),
                "贝丝最终承认南希是对的。".to_owned()
            )
        );
        assert_eq!(sense.examples.len(), 2);

        // The translation-side parentheses must not leak into the definition.
        assert_eq!(group.senses[1].translation, "给予 (权利或特权)");
        assert_eq!(
            group.senses[3].translation,
            "(通过认输而) 使 (比赛、竞赛、争论等) 结束"
        );
    }

    #[test]
    fn parses_grouped_entries_and_skips_pos_headings() {
        let entry = parse_html(include_str!("fixtures/collins_grouped.html")).unwrap();
        assert_eq!(entry.groups.len(), 2);

        let verb = &entry.groups[0];
        assert_eq!(verb.title.as_deref(), Some("VERB USES"));
        assert_eq!(verb.headword, "fast");
        assert_eq!(verb.phonetic.as_deref(), Some("/fæst/"));
        assert_eq!(verb.ranks.as_deref(), Some("CET4 TEM4"));
        assert_eq!(verb.word_forms.as_deref(), Some("(fasting, fasted, fasts)"));
        assert_eq!(verb.senses.len(), 1);
        assert_eq!(verb.senses[0].pos.as_deref(), Some("V-I"));
        assert_eq!(
            verb.senses[0].translation, "禁食 [正式]",
            "register notes follow the translation"
        );
        // Empty example placeholders and inline markup must be handled.
        assert_eq!(
            verb.senses[0].examples,
            [(
                "She fasted for three days.".to_owned(),
                "她禁食了三天。".to_owned()
            )]
        );

        let noun = &entry.groups[1];
        assert_eq!(noun.title.as_deref(), Some("NOUN USES"));
        assert_eq!(noun.phonetic, None);
        assert_eq!(noun.ranks, None);
        assert_eq!(noun.word_forms, None);
        // The bare POS heading `li` is skipped, leaving only the real senses.
        assert_eq!(noun.senses.len(), 2);
        assert_eq!(noun.senses[0].pos.as_deref(), Some("N-COUNT"));
        assert_eq!(noun.senses[0].translation, "禁食期");
        // Cross-reference senses have no Chinese translation or POS.
        assert_eq!(noun.senses[1].pos, None);
        assert_eq!(noun.senses[1].definition, "→ see also breakfast");
        assert_eq!(noun.senses[1].translation, "");
        assert!(noun.senses[1].examples.is_empty());
    }

    #[test]
    fn missing_or_empty_collins_block_is_not_an_error() {
        assert!(parse_html("<div id='phrsListTab'></div>").is_none());
        assert!(
            parse_html("<div id='collinsResult'><div class='collinsToggle'></div></div>").is_none()
        );
        // A block with only a heading has no usable content either.
        assert!(
            parse_html(
                "<div id='collinsResult'><div class='collinsToggle'><div class='wt-container'><h4><span class='title'>fast</span></h4></div></div></div>"
            )
            .is_none()
        );
    }

    #[test]
    fn translation_splitting_handles_cjk_boundaries() {
        assert_eq!(
            split_translation("If you run, you move quickly. 跑"),
            ("If you run, you move quickly.".to_owned(), "跑".to_owned())
        );
        assert_eq!(
            split_translation("To give. (常指不情愿地) 给予"),
            ("To give.".to_owned(), "(常指不情愿地) 给予".to_owned())
        );
        assert_eq!(
            split_translation("→ see also best, better"),
            ("→ see also best, better".to_owned(), String::new())
        );
        assert_eq!(
            split_translation("承认"),
            (String::new(), "承认".to_owned())
        );
    }
}
