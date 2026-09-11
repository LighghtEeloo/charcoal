use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};
use whatlang::Lang;

#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
    vault_dir: PathBuf,
    tmp_dir: PathBuf,
}

/// A versioned cache identity, independent of filesystem path syntax.
#[derive(Clone, Debug)]
pub struct CacheKey(String);

impl CacheKey {
    pub fn new(word: &str, language: Lang, provider: &str, version: u32) -> Self {
        // Length-delimited JSON fields avoid ambiguous concatenated identities.
        let identity = serde_json::to_vec(&(version, provider, language, word))
            .expect("Cache identity fields must serialize");
        Self(format!("v1-{:x}", Sha256::digest(identity)))
    }

    fn from_archive_name(name: &str, suffix: &str) -> io::Result<Self> {
        if let Some(digest) = name.strip_prefix("v1-") {
            if digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Ok(Self(name.to_ascii_lowercase()));
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid cache digest",
            ));
        }
        // Legacy exports contain raw words and have no language metadata.
        let provider = if suffix == "bin" {
            "youdao"
        } else {
            "google-tts"
        };
        Ok(Self::new(name, Lang::Eng, provider, 1))
    }
}

impl Cache {
    pub fn new(cache_dir: PathBuf, vault_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        Self {
            cache_dir,
            vault_dir,
            tmp_dir,
        }
    }

    fn get_file_path(&self, key: &CacheKey, suffix: &'static str) -> io::Result<PathBuf> {
        if !matches!(suffix, "bin" | "mp3") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Unsupported cache format",
            ));
        }
        Ok(self
            .cache_dir
            .join(&key.0[3..5])
            .join(format!("{}.{}", key.0, suffix)))
    }

    pub fn query(&self, key: &CacheKey, suffix: &'static str) -> io::Result<File> {
        File::open(self.get_file_path(key, suffix)?)
    }

    /// Publish a complete file without exposing partial writes to readers.
    pub fn store(&self, key: &CacheKey, suffix: &'static str, bytes: &[u8]) -> io::Result<()> {
        let path = self.get_file_path(key, suffix)?;
        let parent = path.parent().expect("Cache files have a parent directory");
        fs::create_dir_all(parent)?;
        let mut pending = tempfile::NamedTempFile::new_in(parent)?;
        pending.write_all(bytes)?;
        pending.as_file().sync_all()?;
        pending.persist(path).map_err(|err| err.error)?;
        Ok(())
    }

    pub fn show(&self) -> &PathBuf {
        &self.cache_dir
    }

    pub fn clean(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.cache_dir)?;
        fs::remove_dir_all(&self.vault_dir)?;

        Ok(())
    }

    fn tilde_expand(dir: impl AsRef<Path>) -> io::Result<PathBuf> {
        let mut path = (dir.as_ref().iter().take(1))
            .map(|s| -> io::Result<_> {
                if s == "~" {
                    Ok(directories::UserDirs::new()
                        .ok_or(io::Error::from(io::ErrorKind::Unsupported))?
                        .home_dir()
                        .to_path_buf())
                } else {
                    Ok(PathBuf::from(s))
                }
            })
            .collect::<io::Result<PathBuf>>()?;
        for s in dir.as_ref().iter().skip(1) {
            path.push(s)
        }
        Ok(path)
    }

    fn ensure_dir(dir: &PathBuf) -> io::Result<()> {
        if (dir.parent())
            .and_then(|p| if p.exists() { Some(()) } else { None })
            .is_none()
        {
            println!("Parent dir of target not exist.");
            Err(io::Error::from(io::ErrorKind::NotFound))?
        }
        Ok(())
    }

    pub fn import(&self, dir: PathBuf) -> io::Result<()> {
        fs::remove_dir_all(&self.tmp_dir)?;
        fs::create_dir_all(&self.tmp_dir)?;

        let dir = Self::tilde_expand(dir)?;
        Self::ensure_dir(&dir)?;

        let i_file = File::open(dir)?;
        let mut archive = tar::Archive::new(i_file);
        archive.unpack(&self.tmp_dir)?;

        for direntry in fs::read_dir(&self.tmp_dir)? {
            let direntry = direntry?;
            let src = direntry.path();

            let (src_name, src_suffix) = {
                fn split_file_at_dot(file: String) -> Result<(String, String), io::Error> {
                    if let Some((a, b)) = file.rsplit_once('.') {
                        Ok((a.to_owned(), b.to_owned()))
                    } else {
                        Err(io::Error::from(io::ErrorKind::InvalidInput))?
                    }
                }
                src.file_name()
                    .and_then(|s| s.to_str().map(str::to_owned))
                    .map(split_file_at_dot)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))??
            };
            let src_suffix = match src_suffix.as_str() {
                "bin" => "bin",
                "mp3" => "mp3",
                _ => Err(io::Error::from(io::ErrorKind::InvalidInput))?,
            };
            let key = CacheKey::from_archive_name(&src_name, src_suffix)?;
            self.store(&key, src_suffix, &fs::read(src)?)?;
        }
        fs::remove_dir_all(&self.tmp_dir)?;
        Ok(())
    }

    pub fn export(&self, dir: PathBuf) -> io::Result<()> {
        let dir = Self::tilde_expand(dir)?;
        Self::ensure_dir(&dir)?;
        if dir.exists() {
            println!("Target exists.")
        }
        let o_file = File::create(dir)?;
        let mut builder = tar::Builder::new(o_file);
        (self.cache_dir.read_dir()?)
            .flat_map(|sub| -> io::Result<_> {
                let iter = sub?.path().read_dir()?;
                Ok(iter)
            })
            .flatten()
            .flat_map(|file| -> io::Result<_> {
                let p = file?.path();
                Ok(p)
            })
            .try_for_each(|path| -> io::Result<_> {
                builder.append_path_with_name(&path, path.file_name().unwrap())?;
                Ok(())
            })?;
        builder.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn cache(root: &Path) -> Cache {
        for dir in ["cache", "vault", "tmp"] {
            fs::create_dir_all(root.join(dir)).unwrap();
        }
        Cache::new(root.join("cache"), root.join("vault"), root.join("tmp"))
    }

    #[test]
    fn arbitrary_queries_remain_inside_cache() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path());
        for word in [
            "../../outside",
            "/tmp/outside",
            "C:\\outside",
            "hello world",
            "\u{6e05}\u{6670}",
        ] {
            let key = CacheKey::new(word, Lang::Eng, "youdao", 1);
            cache.store(&key, "bin", b"entry").unwrap();
            let path = cache
                .get_file_path(&key, "bin")
                .unwrap()
                .canonicalize()
                .unwrap();
            assert!(path.starts_with(cache.cache_dir.canonicalize().unwrap()));
        }
        assert!(!root.path().join("outside.bin").exists());
    }

    #[test]
    fn keys_separate_languages_providers_and_versions() {
        let base = CacheKey::new("hello", Lang::Eng, "youdao", 1);
        for key in [
            CacheKey::new("hello", Lang::Fra, "youdao", 1),
            CacheKey::new("hello", Lang::Eng, "google-tts", 1),
            CacheKey::new("hello", Lang::Eng, "youdao", 2),
        ] {
            assert_ne!(base.0, key.0);
        }
        assert_eq!(base.0, CacheKey::new("hello", Lang::Eng, "youdao", 1).0);
    }

    #[test]
    fn shorter_writes_replace_the_entire_file() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path());
        let key = CacheKey::new("hello", Lang::Eng, "youdao", 1);
        cache.store(&key, "bin", b"long payload").unwrap();
        cache.store(&key, "bin", b"new").unwrap();
        let mut actual = Vec::new();
        cache
            .query(&key, "bin")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, b"new");
    }

    #[test]
    fn concurrent_writes_publish_complete_files() {
        let root = tempfile::tempdir().unwrap();
        let cache = cache(root.path());
        let key = CacheKey::new("hello", Lang::Eng, "youdao", 1);
        cache.store(&key, "bin", &[0; 8192]).unwrap();
        std::thread::scope(|scope| {
            for value in 1..=4 {
                let cache = &cache;
                let key = &key;
                scope.spawn(move || {
                    for _ in 0..10 {
                        cache.store(key, "bin", &vec![value; 8192]).unwrap();
                    }
                });
            }
            for _ in 0..100 {
                let mut bytes = Vec::new();
                cache
                    .query(&key, "bin")
                    .unwrap()
                    .read_to_end(&mut bytes)
                    .unwrap();
                assert_eq!(bytes.len(), 8192);
                assert!(bytes.iter().all(|b| *b == bytes[0]));
            }
        });
    }

    #[test]
    fn archives_preserve_hashed_identities() {
        let source = tempfile::tempdir().unwrap();
        let target = tempfile::tempdir().unwrap();
        let source_cache = cache(source.path());
        let target_cache = cache(target.path());
        let key = CacheKey::new("hello world", Lang::Fra, "google-tts", 1);
        source_cache.store(&key, "mp3", b"audio").unwrap();
        let archive = source.path().join("export.tar");
        source_cache.export(archive.clone()).unwrap();
        target_cache.import(archive).unwrap();
        let mut actual = Vec::new();
        target_cache
            .query(&key, "mp3")
            .unwrap()
            .read_to_end(&mut actual)
            .unwrap();
        assert_eq!(actual, b"audio");
    }
}
