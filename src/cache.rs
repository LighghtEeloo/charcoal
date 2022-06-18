use crate::WordEntry;
use std::{
    collections::hash_map::DefaultHasher,
    fs::OpenOptions,
    hash::{Hash, Hasher},
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
    vault_dir: PathBuf,
}

#[derive(Debug)]
struct CacheMiss;

impl CacheMiss {
    fn new() -> anyhow::Error {
        anyhow::Error::new(CacheMiss)
    }
}

impl std::fmt::Display for CacheMiss {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for CacheMiss {}

enum CacheFile {
    Normal(u8, String),
    Absurd(u64),
}

impl CacheFile {
    fn str_hash(s: impl AsRef<str>) -> u64 {
        let mut hasher = DefaultHasher::new();
        s.as_ref().hash(&mut hasher);
        hasher.finish()
    }
    fn generate(s: String) -> Self {
        let hash_num = CacheFile::str_hash(&s);
        if s.contains(" ") || !s.is_ascii() {
            CacheFile::Normal((hash_num % 256) as u8, s)
        } else {
            CacheFile::Absurd(hash_num)
        }
    }
    fn consume(self, cache: &Cache) -> PathBuf {
        match self {
            CacheFile::Normal(dir, file) => {
                let mut path = cache.cache_dir.clone();
                path.push(format!("{:02x}/{}.bin", dir, file));
                path
            }
            CacheFile::Absurd(file) => {
                let mut path = cache.vault_dir.clone();
                path.push(format!("{:x}.bin", file));
                path
            }
        }
    }
}

impl Cache {
    pub fn new(cache_dir: PathBuf, vault_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            vault_dir,
        }
    }
    fn get_file_path(&self, word: impl AsRef<str>) -> PathBuf {
        CacheFile::generate(word.as_ref().to_owned()).consume(&self)
    }
    fn read_word_from_file(&self, file: impl Read) -> anyhow::Result<WordEntry> {
        let entry = bincode::deserialize_from(file)?;
        Ok(entry)
    }
    pub fn query(&self, word: impl AsRef<str>) -> anyhow::Result<WordEntry> {
        let file = {
            let path = self.get_file_path(&word);
            OpenOptions::new().read(true).open(path)
        }?;
        self.read_word_from_file(file)
    }
    fn write_word_to_file(&self, file: impl Write, word_entry: WordEntry) -> anyhow::Result<()> {
        bincode::serialize_into(file, &word_entry)?;
        Ok(())
    }
    pub fn store(&mut self, word: impl AsRef<str>, word_query: WordEntry) -> anyhow::Result<()> {
        let file = {
            let path = self.get_file_path(&word);
            OpenOptions::new().create(true).write(true).open(path)?
        };
        self.write_word_to_file(file, word_query)?;
        Ok(())
    }

    pub fn clean(&mut self) -> anyhow::Result<()> {
        todo!("cache cleaning not implemented")
    }
}
