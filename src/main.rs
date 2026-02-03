mod challenge;
mod cli;
mod difficulty;
mod display;

use clap::Parser;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli::run(cli);
}