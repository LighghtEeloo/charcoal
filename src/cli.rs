use clap::{Args, Parser, Subcommand, ValueEnum};

/// A command line dictionary
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn new() -> Command {
        Self::parse().command
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Query words from online or offline
    #[clap(alias = "q")]
    Query(QueryArgs),
    /// Edit the configuration file
    #[clap(alias = "e")]
    Edit,
}

#[derive(Args, Debug)]
pub struct QueryArgs {
    /// The word to be queried
    #[clap(value_parser)]
    pub query: String,
    /// Whether to speak aloud
    #[clap(value_parser, short, long)]
    pub speak: Option<Toggle>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum Toggle {
    /// Yes
    Y,
    /// No
    N,
    /// Toggle
    T,
}

impl Toggle {
    pub fn twitch(self, b: &mut bool) {
        match self {
            Toggle::Y => *b = true,
            Toggle::N => *b = false,
            Toggle::T => *b = !*b,
        }
    }
}
