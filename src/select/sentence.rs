use super::*;

pub struct Sentence;

impl Select for Sentence {
    type Target = Vec<(String, String)>;

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("#bilingual.trans-container li").unwrap();
        Ok(elem.select(&sel).filter_map(|child| {
            Sen::select(child).ok()
        }).collect())
    }
}

struct Sen;

impl Select for Sen {
    type Target = (String, String);

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("p").unwrap();
        let mut iter = elem.select(&sel);

        fn text_and_join(elem: ElementRef, sep: &str) -> String {
            elem.text()
                .filter_map(trim_str)
                .collect::<Vec<String>>()
                .join(sep)
        }

        let ori = text_and_join(iter.next().expect("No ori found in sentence"), " ");
        let trans = text_and_join(iter.next().expect("No trans found in sentence"), "");
        Ok((ori, trans))
    }
}
