use clap::Parser;

/// a command line dictionary
#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct Args {
    /// The word to be queried
    #[clap(value_parser)]
    pub query_word: String,
    /// Whether to speak aloud
    #[clap(short, long, value_parser, default_value_t = false)]
    pub speak: bool,
}
