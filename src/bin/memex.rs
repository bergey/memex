// memex cli

use clap::Parser;
use std::io;

use memex::tag::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command()]
    query: String,
    // #[arg(long="library")]
    // libraries: Vec<String>
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    Ok(())
}
