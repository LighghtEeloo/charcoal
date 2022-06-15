//! Charcoal, a command line dictionary
//! 
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

pub mod config;
pub mod display;
pub mod query;
pub mod speech;
pub mod cli;

pub use config::{Config, Toggle};
pub use query::Word;
pub use speech::speak;