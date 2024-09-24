#![allow(dead_code)]
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "sqliters", long_about = None)]
pub struct Args {
    /// Input sqlite file
    #[clap(long, short)]
    pub input: String,
}

impl Args {
    pub fn from() -> Args {
        Args::parse()
    }
}
