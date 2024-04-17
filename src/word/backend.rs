mod youdict;

use crate::{Acquire, Cache, ExactQuery, Question, SingleEntry};

pub struct QueryCache<'a> {
    cache: &'a Cache,
}

impl<'a> QueryCache<'a> {
    pub fn new(cache: &'a Cache) -> Self {
        Self { cache }
    }
}

impl<'a> Acquire for QueryCache<'a> {
    type WordQuery = ExactQuery;
    type WordEntry = SingleEntry;
    fn acquire(self, word_query: &ExactQuery) -> anyhow::Result<SingleEntry> {
        if word_query.refresh {
            anyhow::bail!("Force refreshing the cache.")
        }
        let file = self.cache.query(word_query.word(), "bin")?;
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
}

pub struct QueryYoudict;

impl QueryYoudict {
    pub fn new() -> Self {
        Self
    }
    pub fn query_and_store(
        self, word_query: &ExactQuery, cache: &Cache,
    ) -> anyhow::Result<SingleEntry> {
        let word_entry = self.acquire(word_query)?;
        let file = cache.store(word_query.word(), "bin")?;
        bincode::serialize_into(file, &word_entry)?;
        Ok(word_entry)
    }
}
