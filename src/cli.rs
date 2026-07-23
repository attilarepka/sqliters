use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "sqliters", long_about = None)]
pub struct Args {
    /// Input sqlite file
    #[arg(index = 1)]
    pub input: String,
}
