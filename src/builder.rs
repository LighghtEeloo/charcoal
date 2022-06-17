use crate::{Cache, Config};
use directories_next::ProjectDirs;
use log::info;
use std::fs;

pub struct AppDataBuilder {
    project_dirs: ProjectDirs,
    config_file: &'static str,
    cache_file: &'static str,
    cache_dir: &'static str,
}

impl AppDataBuilder {
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

impl AppDataBuilder {
    pub fn config(&self) -> anyhow::Result<Config> {
        let config_path = {
            let mut config_path = self.project_dirs.config_dir().to_path_buf();
            fs::create_dir_all(&config_path)?;
            config_path.push(self.config_file);
            config_path
        };

        Config::of_file(&config_path).map_or_else(
            |_err| -> anyhow::Result<Config> {
                info!(
                    "Creating new configuration file at: \n\t{}",
                    config_path.display()
                );
                let config = Config::default();
                config.to_file(&config_path)?;
                Ok(config)
            },
            |config| Ok(config),
        )
    }

    pub fn cache(&self) -> anyhow::Result<Cache> {
        let (cache_file, cache_dir) = {
            let mut cache_file = self.project_dirs.cache_dir().to_path_buf();
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
