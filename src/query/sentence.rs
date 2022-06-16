use super::*;

pub struct Sentence;

impl Select for Sentence {
    type Target = Vec<(String, String)>;

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("#bilingual.trans-container li").unwrap();
        let mut vec = Vec::new();
        for child in elem.select(&sel) {
            vec.push(Sen::select(child)?)
        }
        Ok(vec)
    }
}

struct Sen;

impl Select for Sen {
    type Target = (String, String);

    fn select(elem: ElementRef) -> anyhow::Result<Self::Target> {
        let sel = Selector::parse("p").unwrap();
        let mut iter = elem.select(&sel);
        let ori = iter
            .next()
            .expect("No ori found in sentence")
            .text()
            .filter_map(trim_str)
            .collect::<Vec<String>>()
            .join(" ");
        let trans = iter
            .next()
            .expect("No trans found in sentence")
            .text()
            .filter_map(trim_str)
            .collect::<Vec<String>>()
            .join("");
        Ok((ori, trans))
    }
}
