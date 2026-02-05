use std::env;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::challenge::load_daily_challenge;
use crate::codegen::generate_scaffold;
use crate::config;
use crate::difficulty::Difficulty;
use crate::display::display_challenge;
use crate::language::Language;
use crate::project;
use crate::signature::parse_signature;
use crate::testrun;

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
    Difficulty {
        #[arg(value_enum)]
        level: Option<Difficulty>,
    },
    Test,
    Submit,
}

pub fn run(cli: Cli) {
    match cli.command {
        None => show_challenge(),
        Some(Commands::Init { language }) => init_challenge(language),
        Some(Commands::Difficulty { level }) => handle_difficulty(level),
        Some(Commands::Test) => test_solution(),
        Some(Commands::Submit) => submit_solution(),
    }
}

fn show_challenge() {
    let user_config = config::load_config();

    match load_daily_challenge(user_config.difficulty) {
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
    let user_config = config::load_config();

    let challenge = match load_daily_challenge(user_config.difficulty) {
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

    match generate_scaffold(&challenge, &sig, language, user_config.difficulty, &output_dir) {
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

fn print_run_instructions(_language: Language, dir_name: &str) {
    println!("To get started:");
    println!();
    println!("  cd {}", dir_name);
    println!("  ./setup.sh");
}

fn handle_difficulty(level: Option<Difficulty>) {
    let mut user_config = config::load_config();

    match level {
        None => {
            println!("Current difficulty: {}", user_config.difficulty.display_name());
            println!("BOSS Score: {}", user_config.boss_score);
            println!("Challenges completed: {}", user_config.challenges_completed);
            println!();
            println!("BOSS Score Multipliers:");
            println!("  Easy:    1.0x");
            println!("  Medium:  1.5x");
            println!("  Hard:    2.5x");
            println!("  Extreme: 4.0x");
            println!();
            println!("To change: codle difficulty <level>");
        }
        Some(new_level) => {
            let old_level = user_config.difficulty;
            if old_level == new_level {
                println!("Difficulty is already set to {}", new_level.display_name());
                return;
            }

            user_config.difficulty = new_level;
            if let Err(e) = config::save_config(&user_config) {
                eprintln!("Failed to save config: {}", e);
                std::process::exit(1);
            }

            println!(
                "Difficulty changed from {} to {}",
                old_level.display_name(),
                new_level.display_name()
            );
            println!(
                "You'll now earn {} points per completed challenge!",
                new_level.points_for_completion()
            );
        }
    }
}

fn test_solution() {
    let current_dir = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Failed to get current directory: {}", e);
        std::process::exit(1);
    });

    // Load project metadata
    let metadata = match project::load(&current_dir) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    println!(
        "Running tests for {} ({})...",
        metadata.challenge_name,
        metadata.language.display_name()
    );
    println!();

    // Run tests
    let summary = match testrun::run_tests(metadata.language) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run tests: {}", e);
            std::process::exit(1);
        }
    };

    // Display results
    println!("========================================");
    if summary.failed == 0 {
        println!(
            "{}/{} tests passed",
            summary.passed, summary.total
        );
    } else {
        println!(
            "{}/{} tests passed - {} failed",
            summary.passed, summary.total, summary.failed
        );
    }
    println!("========================================");

    if summary.failed > 0 {
        std::process::exit(1);
    }
}

fn submit_solution() {
    println!("Submitting solution...");
    println!("(Not yet implemented)");
}
