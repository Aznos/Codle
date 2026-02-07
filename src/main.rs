mod cli;
mod display;
mod lang;
mod models;

use clap::Parser;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli::run(cli);
}
