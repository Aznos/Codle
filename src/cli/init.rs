use std::path::PathBuf;

use crate::models::{load_daily_challenge, parse_signature, Language, config};
use crate::lang::generate_scaffold;

pub fn init_challenge(language: Language) {
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
