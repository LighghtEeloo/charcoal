pub mod config;
pub mod display;
pub mod query;
pub mod speech;

pub use config::{Config, Toggle};
pub use query::Word;
pub use speech::speak;