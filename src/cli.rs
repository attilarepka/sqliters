#![allow(dead_code)]
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "sqliters", long_about = None)]
pub struct Args {
    /// Input archive file
    #[clap(long, short)]
    pub input_file: String,
}

impl Args {
    pub async fn from() -> Args {
        Args::parse()
    }
}
