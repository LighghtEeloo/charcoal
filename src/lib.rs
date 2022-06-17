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
pub mod speech;

pub use builder::AppDataBuilder;
pub use cache::Cache;
pub use cli::Args;
pub use config::{Config, Toggle};
pub use query::{CacheQuery, WebQuery, WordQuery};
pub use speech::Speech;
