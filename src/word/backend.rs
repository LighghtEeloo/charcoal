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
        let mut file = self.cache.query(word_query.word(), "bin")?;
        let mut buf = Vec::new();
        use std::io::Read;
        file.read_to_end(&mut buf)?;
        let entry = wincode::deserialize_from(buf.as_slice())?;
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
        let mut file = cache.store(word_query.word(), "bin")?;
        let mut buf = Vec::new();
        wincode::serialize_into(&mut buf, &word_entry)?;
        use std::io::Write;
        file.write_all(&buf)?;

        Ok(word_entry)
    }
}
