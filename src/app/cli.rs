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
    #[clap(aliases = &["q", "search", "s"])]
    Query(QueryArgs),
    /// Edit the configuration file
    #[clap(aliases = &["e", "config"])]
    Edit(EditArgs),
    /// Clean cache
    #[clap(aliases = &["c"])]
    Clean,
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
    /// Be concise
    #[clap(value_parser, short, long)]
    pub concise: bool,
    /// Whether to be concise
    #[clap(value_parser, long)]
    pub concise_as: Option<Toggle>,
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
