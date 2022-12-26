use super::{FromCache, Query, WordEntry, WordQuery};

impl<'a> Query for FromCache<'a> {
    fn query(&mut self, word_query: &WordQuery) -> anyhow::Result<WordEntry> {
        let file = self.cache.query(word_query.word(), "bin")?;
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
}
