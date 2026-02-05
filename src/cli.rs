use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::challenge::load_daily_challenge;
use crate::codegen::generate_scaffold;
use crate::difficulty::Difficulty;
use crate::display::display_challenge;
use crate::language::Language;
use crate::signature::parse_signature;

#[derive(Parser)]
#[command(name = "codle")]
#[command(about = "Daily coding challenges", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(value_enum)]
        language: Language,
    },
    Test,
    Submit,
}

pub fn run(cli: Cli) {
    match cli.command {
        None => show_challenge(),
        Some(Commands::Init { language }) => init_challenge(language),
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

fn init_challenge(language: Language) {
    let difficulty = Difficulty::Easy;

    let challenge = match load_daily_challenge(difficulty) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load challenge: {}", e);
            std::process::exit(1);
        }
    };

    let sig = match parse_signature(&challenge.function_signature) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse function signature: {}", e);
            std::process::exit(1);
        }
    };

    // Create output directory from challenge name (snake_case)
    let dir_name = challenge
        .name
        .to_lowercase()
        .replace(' ', "_")
        .replace(|c: char| !c.is_alphanumeric() && c != '_', "");
    let output_dir = PathBuf::from(&dir_name);

    if output_dir.exists() {
        eprintln!(
            "Directory '{}' already exists. Remove it first or use a different location.",
            dir_name
        );
        std::process::exit(1);
    }

    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create directory '{}': {}", dir_name, e);
        std::process::exit(1);
    }

    match generate_scaffold(&challenge, &sig, language, &output_dir) {
        Ok(()) => {
            println!(
                "Initialized {} scaffold for '{}' in ./{}/",
                language.display_name(),
                challenge.name,
                dir_name
            );
            println!();
            print_run_instructions(language, &dir_name);
        }
        Err(e) => {
            eprintln!("Failed to generate scaffold: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_run_instructions(language: Language, dir_name: &str) {
    println!("To run your solution:");
    println!();
    match language {
        Language::Rs => {
            println!("  cd {}", dir_name);
            println!("  cargo run");
        }
        Language::Py => {
            println!("  cd {}", dir_name);
            println!("  ./setup.sh          # Create virtual environment (first time)");
            println!("  source venv/bin/activate");
            println!("  python solution.py");
        }
        Language::Kt => {
            println!("  cd {}", dir_name);
            println!("  gradle run          # Or: ./gradlew run");
        }
        Language::Java => {
            println!("  cd {}", dir_name);
            println!("  gradle run          # Or: ./gradlew run");
        }
        Language::C => {
            println!("  cd {}", dir_name);
            println!("  make && ./solution");
        }
        Language::Cpp => {
            println!("  cd {}", dir_name);
            println!("  make && ./solution");
        }
    }
}

fn test_solution() {
    println!("Testing solution...");
    println!("(Not yet implemented)");
}

fn submit_solution() {
    println!("Submitting solution...");
    println!("(Not yet implemented)");
}
