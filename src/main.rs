mod challenge;
mod cli;
mod codegen;
mod config;
mod difficulty;
mod display;
mod language;
mod project;
mod signature;
mod testgen;
mod testrun;

use clap::Parser;
use crate::cli::Cli;

fn main() {
    let cli = Cli::parse();
    cli::run(cli);
}