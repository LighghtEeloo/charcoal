//! Charcoal, a command line dictionary
//!
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

pub mod app;
pub mod word;

pub use app::builder::AppBuilder;
pub use app::cache::Cache;
pub use app::cli::{Cli, Command};
pub use app::config::Config;
pub use word::speech::Speech;
pub use word::{WordEntry, WordQuery};
