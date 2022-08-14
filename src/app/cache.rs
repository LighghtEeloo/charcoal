use std::{
    collections::hash_map::DefaultHasher,
    fs::{self, File, OpenOptions},
    hash::{Hash, Hasher},
    io,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Cache {
    cache_dir: PathBuf,
    vault_dir: PathBuf,
    tmp_dir: PathBuf,
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
            CacheFile::Absurd(hash_num)
        } else {
            CacheFile::Normal((hash_num % 256) as u8, s)
        }
    }
    fn consume(self, cache: &Cache, suffix: &'static str) -> io::Result<PathBuf> {
        match self {
            CacheFile::Normal(dir, file) => {
                let mut path = cache.cache_dir.clone();
                path.push(format!("{:02x}", dir));
                fs::create_dir_all(&path)?;
                path.push(format!("{}.{}", file, suffix));
                Ok(path)
            }
            CacheFile::Absurd(file) => {
                let mut path = cache.vault_dir.clone();
                path.push(format!("{:x}.{}", file, suffix));
                Ok(path)
            }
        }
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

    fn get_file_path(&self, word: impl AsRef<str>, suffix: &'static str) -> io::Result<PathBuf> {
        CacheFile::generate(word.as_ref().to_owned()).consume(&self, suffix)
    }

    pub fn query(&self, word: impl AsRef<str>, suffix: &'static str) -> io::Result<File> {
        let path = self.get_file_path(&word, suffix)?;
        let file = OpenOptions::new().read(true).open(path)?;
        Ok(file)
    }

    pub fn store(&self, word: impl AsRef<str>, suffix: &'static str) -> io::Result<File> {
        let path = self.get_file_path(&word, suffix)?;
        let file = OpenOptions::new().create(true).write(true).open(path)?;
        Ok(file)
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
        let mut path = (dir.as_ref().into_iter().take(1))
            .map(|s| -> io::Result<_> {
                if s == "~" {
                    Ok(directories_next::UserDirs::new()
                        .ok_or(io::Error::from(io::ErrorKind::Unsupported))?
                        .home_dir()
                        .to_path_buf())
                } else {
                    Ok(PathBuf::from(s))
                }
            })
            .collect::<io::Result<PathBuf>>()?;
        for s in dir.as_ref().into_iter().skip(1) {
            path.push(s)
        }
        Ok(path)
    }

    fn ensure_dir(dir: &PathBuf) -> io::Result<()> {
        if (dir.parent())
            .map(|p| if p.exists() { Some(()) } else { None })
            .flatten()
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
                    .map(|s| s.to_str().unwrap().to_owned())
                    .map(split_file_at_dot)
                    .ok_or(io::Error::from(io::ErrorKind::InvalidInput))??
            };
            let src_suffix = match src_suffix.as_str() {
                "bin" => "bin",
                "mp3" => "mp3",
                _ => Err(io::Error::from(io::ErrorKind::InvalidInput))?,
            };
            let mut src = OpenOptions::new().read(true).open(src)?;
            let mut dest = self.store(src_name, src_suffix)?;
            io::copy(&mut src, &mut dest)?;
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
        directories_next::UserDirs::new().unwrap().home_dir();
        Ok(())
    }
}
