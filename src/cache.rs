use std::{
    collections::hash_map::DefaultHasher,
    fs::{File, OpenOptions},
    hash::{Hash, Hasher},
    io,
    path::PathBuf,
};

#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
    vault_dir: PathBuf,
}

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
    fn consume(self, cache: &Cache, suffix: &'static str) -> PathBuf {
        match self {
            CacheFile::Normal(dir, file) => {
                let mut path = cache.cache_dir.clone();
                path.push(format!("{:02x}", dir));
                path.push(format!("{}.{}", file, suffix));
                path
            }
            CacheFile::Absurd(file) => {
                let mut path = cache.vault_dir.clone();
                path.push(format!("{:x}.{}", file, suffix));
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
    fn get_file_path(&self, word: impl AsRef<str>, suffix: &'static str) -> PathBuf {
        CacheFile::generate(word.as_ref().to_owned()).consume(&self, suffix)
    }
    pub fn query(&self, word: impl AsRef<str>, suffix: &'static str) -> io::Result<File> {
        let path = self.get_file_path(&word, suffix);
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(file)
    }
    pub fn store(&self, word: impl AsRef<str>, suffix: &'static str) -> io::Result<File> {
        let path = self.get_file_path(&word, suffix);
        let file = OpenOptions::new().create(true).write(true).open(path)?;
        Ok(file)
    }

    pub fn clean(&mut self) -> io::Result<()> {
        todo!("cache cleaning not implemented")
    }
}
