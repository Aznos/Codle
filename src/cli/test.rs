use std::env;

use crate::models::project;
use crate::lang::run_tests;

pub fn test_solution() {
    let current_dir = env::current_dir().unwrap_or_else(|e| {
        eprintln!("Failed to get current directory: {}", e);
        std::process::exit(1);
    });

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

    let summary = match run_tests(metadata.language) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run tests: {}", e);
            std::process::exit(1);
        }
    };

    if !summary.output.trim().is_empty() {
        println!("{}", summary.output.trim());
        println!();
    }

    println!("========================================");
    if summary.total == 0 {
        println!("No test results found. Check the output above for errors.");
    } else if summary.failed == 0 {
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

    if summary.total == 0 || summary.failed > 0 {
        std::process::exit(1);
    }
}
