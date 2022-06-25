use crate::{Cache, Config};
use directories_next::ProjectDirs;
use std::{fs, io, path::PathBuf};

pub struct AppBuilder {
    project_dirs: ProjectDirs,
    config_file: &'static str,
    cache_dir: &'static str,
    vault_dir: &'static str,
    tmp_dir: &'static str,
}

impl AppBuilder {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("", "LitiaEeloo", "Charcoal")
            .expect("No valid config directory fomulated");
        Self {
            project_dirs,
            config_file: "config.toml",
            cache_dir: "cache",
            vault_dir: "vault",
            tmp_dir: "tmp",
        }
    }
}

impl AppBuilder {
    fn config_path(&self) -> io::Result<PathBuf> {
        let mut config_path = self.project_dirs.config_dir().to_owned();
        fs::create_dir_all(&config_path)?;
        config_path.push(self.config_file);
        Ok(config_path)
    }
    pub fn config(&self) -> anyhow::Result<Config> {
        Config::of_file(self.config_path()?)
            .or_else(|_err| -> anyhow::Result<Config> { self.config_fresh() })
    }
    pub fn config_fresh(&self) -> anyhow::Result<Config> {
        let config_path = self.config_path()?;
        log::info!(
            "Creating fresh configuration file at: \n\t{}",
            config_path.display()
        );
        let config = Config::new(config_path);
        config.to_file()?;
        Ok(config)
    }

    pub fn cache(&self) -> anyhow::Result<Cache> {
        let ensure = |dir| -> io::Result<PathBuf> {
            let mut pathbuf = self.project_dirs.cache_dir().to_owned();
            pathbuf.push(dir);
            fs::create_dir_all(&pathbuf)?;
            Ok(pathbuf)
        };
        let cache_dir = ensure(self.cache_dir)?;
        let vault_dir = ensure(self.vault_dir)?;
        let tmp_dir = ensure(self.tmp_dir)?;

        let cache = Cache::new(cache_dir, vault_dir, tmp_dir);
        Ok(cache)
    }
}
