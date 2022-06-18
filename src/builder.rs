use crate::{Cache, Config};
use directories_next::ProjectDirs;
use log::info;
use std::{fs, io, path::PathBuf};

pub struct AppBuilder {
    project_dirs: ProjectDirs,
    config_file: &'static str,
    cache_file: &'static str,
    cache_dir: &'static str,
}

impl AppBuilder {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("", "LitiaEeloo", "Charcoal")
            .expect("No valid config directory fomulated");
        Self {
            project_dirs,
            config_file: "config.toml",
            cache_file: "cache.json",
            cache_dir: "cache",
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
        Config::of_file(self.config_path()?).map_or_else(
            |_err| -> anyhow::Result<Config> { self.config_fresh() },
            |config| Ok(config),
        )
    }
    pub fn config_fresh(&self) -> anyhow::Result<Config> {
        let config_path = self.config_path()?;
        info!(
            "Creating new configuration file at: \n\t{}",
            config_path.display()
        );
        let config = Config::new(config_path);
        config.to_file()?;
        Ok(config)
    }

    pub fn cache(&self) -> anyhow::Result<Cache> {
        let (cache_file, cache_dir) = {
            let mut cache_file = self.project_dirs.cache_dir().to_owned();
            let mut cache_dir = cache_file.clone();
            // file path is ensured by dir
            cache_file.push(self.cache_file);
            cache_dir.push(self.cache_dir);
            fs::create_dir_all(&cache_dir)?;
            (cache_file, cache_dir)
        };

        let mut cache = Cache::new(cache_file.clone(), cache_dir);
        if let Err(_) = cache.of_file() {
            info!(
                "Potentially creating new cache file at: \n\t{}",
                cache_file.display()
            );
        }
        Ok(cache)
    }
}
