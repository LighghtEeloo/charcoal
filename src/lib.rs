//! Charcoal, a command line dictionary
//!
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

pub mod cli;
pub mod config;
pub mod display;
pub mod query;
pub mod speech;

pub use config::{Config, ConfigBuilder, Toggle};
pub use query::Word;
pub use speech::speak;
