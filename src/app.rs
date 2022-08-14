pub mod builder;
pub mod cache;
pub mod cli;
pub mod config;

use super::*;

pub struct App {}

impl App {
    pub async fn main() -> anyhow::Result<()> {
        env_logger::init();

        match Cli::new() {
            Commands::Query(args) => App::query(args).await,
            Commands::Edit(args) => App::edit(args).await,
            Commands::Cache { commands } => App::cache(commands).await,
        }
    }

    pub async fn query(mut args: cli::QueryArgs) -> anyhow::Result<()> {
        let app_builder = AppBuilder::new();

        let mut config = app_builder.config()?;
        let cache = app_builder.cache()?;

        config.apply(&mut args);

        let word_query = {
            let word_query = WordQuery::new(args.query());
            if let Some(word_query) = word_query {
                word_query
            } else {
                println!("Invalid input.");
                return Ok(());
            }
        };

        let word_speech = Speech::query(&word_query, &cache, config.speak);
        let word_entry = WordEntry::query(&word_query, &cache).await?;

        if word_entry.is_empty() {
            println!("Word not found.");
            return Ok(());
        }

        word_entry.display(&word_query, &config);
        if let Err(err) = word_speech.await {
            log::error!("An error occured in speech module: {:?}.", err)
        }

        Ok(())
    }

    pub async fn edit(args: cli::EditArgs) -> anyhow::Result<()> {
        use std::process::Command;

        let editor = std::env::var("EDITOR").or_else(|err| {
            println!("Please set $EDITOR to your preferred editor.");
            Err(err)
        })?;
        let config_path = {
            let app_builder = AppBuilder::new();
            if args.reset {
                app_builder.config_fresh()?
            } else {
                app_builder.config()?
            }
            .path
        };

        let mut child = Command::new(editor).args([config_path]).spawn()?;
        child.wait()?;
        Ok(())
    }

    pub async fn cache(cmds: cli::CacheCmds) -> anyhow::Result<()> {
        let cache = AppBuilder::new().cache()?;
        match cmds {
            cli::CacheCmds::Show => {
                println!("{}", cache.show().display());
            }
            cli::CacheCmds::Clean => {
                cache.clean()?;
            }
            cli::CacheCmds::Import { dir } => {
                log::info!("Importing:\n\t<== {}", dir.display());
                cache.import(dir)?;
            }
            cli::CacheCmds::Export { dir } => {
                log::info!("Exporting:\n\t==> {}", dir.display());
                cache.export(dir)?;
            }
        }
        Ok(())
    }
}
