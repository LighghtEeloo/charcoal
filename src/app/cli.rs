pub mod __uses {
    pub use clap::{Args, Parser, Subcommand, ValueEnum};
    pub use std::path::PathBuf;
}
use __uses::*;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub commands: Commands,
}

impl Cli {
    pub fn new() -> Commands {
        Self::parse().commands
    }
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Query words from online or offline
    #[clap(aliases = &["q", "search", "s"])]
    Query(QueryArgs),
    /// Edit the configuration file
    #[clap(aliases = &["e", "config"])]
    Edit(EditArgs),
    /// Cache commands
    #[clap(aliases = &["c"])]
    Cache {
        #[clap(subcommand)]
        commands: CacheCmds,
    },
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// The word to be queried
    #[clap(value_parser)]
    pub query: Vec<String>,
    /// Speak aloud
    #[clap(value_parser, short, long)]
    pub speak: bool,
    /// Mute (overloads speak)
    #[clap(value_parser, short = 'q', long)]
    pub mute: bool,
    /// Whether to speak aloud
    #[clap(value_parser, long)]
    pub speak_as: Option<Toggle>,
    /// Whether to refresh cache
    #[clap(value_parser, long)]
    pub refresh: bool,
    /// Be concise
    #[clap(value_parser, short, long)]
    pub concise: bool,
    /// Whether to be concise
    #[clap(value_parser, long)]
    pub concise_as: Option<Toggle>,
    /// Enable all content sections
    #[clap(short, long, conflicts_with = "sections")]
    pub all: bool,
    /// Content sections to display, in this order (overrides the configuration)
    #[clap(long, value_enum, value_delimiter = ',')]
    pub sections: Option<Vec<SectionArg>>,
    /// Output format
    #[clap(long, value_enum, default_value = "human")]
    pub format: Format,
}

/// A content section selectable from the command line.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum SectionArg {
    /// Youdao's basic translation block
    Youdao,
    /// Collins English-Chinese dictionary block
    Collins,
}

/// How the query result is printed.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Format {
    /// Human readable, colorized output
    #[default]
    Human,
    /// Machine readable JSON for the selected sections
    Json,
}

impl QueryArgs {
    pub fn query(&self) -> String {
        self.query.join(" ")
    }
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// A fresh start
    #[clap(value_parser, long)]
    pub reset: bool,
}

#[derive(Subcommand, Debug)]
pub enum CacheCmds {
    /// Show cache location
    #[clap(aliases = &["ls"])]
    Show,
    /// Clean cache
    #[clap(aliases = &["destroy"])]
    Clean,
    /// Import
    #[clap(aliases = &["in"])]
    Import {
        #[clap(value_parser)]
        dir: PathBuf,
    },
    /// Export
    #[clap(aliases = &["out"])]
    Export {
        #[clap(value_parser)]
        dir: PathBuf,
    },
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Toggle {
    /// True
    True,
    /// False
    False,
    /// Flip
    Flip,
}

impl Toggle {
    pub fn twitch(&self, b: &mut bool) {
        match self {
            Toggle::True => *b = true,
            Toggle::False => *b = false,
            Toggle::Flip => *b = !*b,
        }
    }
    pub fn counter_twitch(&self, b: &mut bool) {
        match self {
            Toggle::True => *b = false,
            Toggle::False => *b = true,
            Toggle::Flip => *b = !*b,
        }
    }
}
