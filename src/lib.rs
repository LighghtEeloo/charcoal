//! Charcoal, a command line dictionary
//!
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

#![allow(dead_code)]

pub mod builder;
pub mod cache;
pub mod cli;
pub mod config;
pub mod display;
pub mod query;
pub mod select;
pub mod speech;
pub mod word;

pub use builder::AppDataBuilder;
pub use cache::Cache;
pub use cli::{Cli, Command};
pub use config::Config;
pub use select::Select;
pub use speech::Speech;
pub use word::WordEntry;
