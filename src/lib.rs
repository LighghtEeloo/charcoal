//! Charcoal, a command line dictionary
//!
//! Charcoal uses youdao dict api and google speech. Inspired by wudao-dict.

pub mod app;
pub mod word;

pub use app::{
    builder::AppBuilder,
    cache::Cache,
    cli::{Cli, Commands},
    config::Config,
    App,
};
pub use word::{
    frontend::{ExactQuery, SingleEntry},
    speech::Speech,
    Acquire, Answer, PPrint, Question,
};
