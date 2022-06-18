use clap::{Args, Parser, Subcommand, ValueEnum};

/// A command line dictionary
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn new() -> Self {
        Self::parse()
    }
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Query words from online or offline
    #[clap(alias = "q")]
    Query(CmdQuery),
}

#[derive(Args, Debug)]
pub struct CmdQuery {
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
