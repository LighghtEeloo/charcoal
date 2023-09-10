//! Charcoal, a command line dictionary
//!
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

pub mod app;
pub mod word;
pub mod suggestion;

pub use app::{
    builder::AppBuilder,
    cache::Cache,
    cli::{Cli, Commands},
    config::Config,
    App,
};
pub use word::{
    frontend::SingleEntry, speech::Speech, Acquire, Answer, ExactQuery, PPrint, Question,
};
pub use suggestion::Suggestion;