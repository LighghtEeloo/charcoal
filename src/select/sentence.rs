use super::*;

pub struct Sentence;

impl Select for Sentence {
    type Target = Vec<(String, String)>;

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("#bilingual.trans-container li").unwrap();
        Ok(elem
            .select(&sel)
            .filter_map(|child| Sen::select(child).ok())
            .collect())
    }
}

struct Sen;
const PUNCTUATORS: &[char; 10] = &[
    '.',
    ',',
    '\"',
    '\'',
    '?',
    '!',
    ':',
    '-',
    '<',
    '>',
];

impl Select for Sen {
    type Target = (String, String);

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("p").unwrap();
        let mut iter = elem.select(&sel);

        fn text_to_vec(elem: ElementRef) -> Vec<String> {
            elem.text().filter_map(trim_str).collect()
        }

        let mut ori = String::new();
        let mut ori_iter = text_to_vec(iter.next().expect("No ori found in sentence")).into_iter();
        if let Some(s) = ori_iter.next() {
            ori.push_str(&s)
        }
        for mut s in ori_iter {
            if !s.starts_with(PUNCTUATORS) {
                s.insert(0, ' ')
            } 
            ori.push_str(&s)
        }

        let trans = text_to_vec(iter.next().expect("No trans found in sentence")).join("");
        Ok((ori, trans))
    }
}
