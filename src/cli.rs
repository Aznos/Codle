use clap::{Parser, Subcommand};

use crate::challenge::load_daily_challenge;
use crate::difficulty::Difficulty;
use crate::display::display_challenge;

#[derive(Parser)]
#[command(name = "codle")]
#[command(about = "Daily coding challenges", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Test,
    Submit,
}

pub fn run(cli: Cli) {
    match cli.command {
        None => show_challenge(),
        Some(Commands::Init) => init_challenge(),
        Some(Commands::Test) => test_solution(),
        Some(Commands::Submit) => submit_solution(),
    }
}

fn show_challenge() {
    let difficulty = Difficulty::Easy;

    match load_daily_challenge(difficulty) {
        Ok(challenge) => {
            display_challenge(&challenge);
        }
        Err(e) => {
            eprintln!("Failed to load challenge: {}", e);
            std::process::exit(1);
        }
    }
}

fn init_challenge() {
    println!("Initializing daily challenge...");
    println!("(Not yet implemented)");
}

fn test_solution() {
    println!("Testing solution...");
    println!("(Not yet implemented)");
}

fn submit_solution() {
    println!("Submitting solution...");
    println!("(Not yet implemented)");
}
