use super::{ExactQuery, Query, QueryCache, SingleEntry};

impl<'a> Query for QueryCache<'a> {
    type WordQuery = ExactQuery;
    type WordEntry = SingleEntry;
    fn query(&mut self, word_query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        let file = self.cache.query(word_query.word(), "bin")?;
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
}
