use std::env;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use clap::{Parser, Subcommand};

use crate::challenge::load_daily_challenge;
use crate::codegen::generate_scaffold;
use crate::config;
use crate::difficulty::{calculate_boss_score, Difficulty};
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
    Info,
}

pub fn run(cli: Cli) {
    match cli.command {
        None => show_challenge(),
        Some(Commands::Init { language }) => init_challenge(language),
        Some(Commands::Difficulty { level }) => handle_difficulty(level),
        Some(Commands::Test) => test_solution(),
        Some(Commands::Submit) => submit_solution(),
        Some(Commands::Info) => generic_info()
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
            println!("Current streak: {} day(s)", user_config.current_streak);
            println!("Longest streak: {} day(s)", user_config.longest_streak);
            println!();
            println!("BOSS Score = challenge_difficulty + tier_bonus + streak_bonus");
            println!();
            println!("Tier bonuses:");
            println!("  Easy:    +0");
            println!("  Medium:  +1");
            println!("  Hard:    +2");
            println!("  Extreme: +3");
            println!();
            println!("Streak bonus: +1 per consecutive day (max +5)");
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
                "Tier bonus is now +{} per challenge",
                new_level.tier_offset()
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

    // Show raw output
    if !summary.output.trim().is_empty() {
        println!("{}", summary.output.trim());
        println!();
    }

    // Display results
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

fn submit_solution() {
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

    // Check if already completed today
    let mut user_config = config::load_config();
    let today = Local::now().format("%Y-%m-%d").to_string();

    if let Some(ref last_date) = user_config.last_completed_date {
        if last_date == &today {
            println!("You've already completed today's challenge!");
            println!();
            println!("Come back tomorrow for a new challenge.");
            println!("BOSS Score: {} | Challenges completed: {}",
                user_config.boss_score, user_config.challenges_completed);
            return;
        }
    }

    // Run tests
    println!(
        "Running tests for {} ({})...",
        metadata.challenge_name,
        metadata.language.display_name()
    );
    println!();

    let summary = match testrun::run_tests(metadata.language) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to run tests: {}", e);
            std::process::exit(1);
        }
    };

    // Show raw output
    if !summary.output.trim().is_empty() {
        println!("{}", summary.output.trim());
        println!();
    }

    if summary.total == 0 {
        println!("========================================");
        println!("No test results found. Check the output above for errors.");
        println!("========================================");
        println!();
        println!("Submission rejected: could not verify tests.");
        std::process::exit(1);
    }

    if summary.failed > 0 {
        println!("========================================");
        println!(
            "{}/{} tests passed - {} failed",
            summary.passed, summary.total, summary.failed
        );
        println!("========================================");
        println!();
        println!("Submission rejected: all tests must pass before submitting.");
        std::process::exit(1);
    }

    // All tests passed - compute streak
    let yesterday = (Local::now() - chrono::Duration::days(1))
        .format("%Y-%m-%d")
        .to_string();

    let streak = if let Some(ref last_date) = user_config.last_completed_date {
        if last_date == &yesterday {
            user_config.current_streak + 1
        } else {
            1
        }
    } else {
        1
    };

    let streak_bonus = streak.min(5);
    let points = calculate_boss_score(
        metadata.challenge_difficulty,
        &metadata.difficulty,
        streak,
    );

    user_config.boss_score += points;
    user_config.challenges_completed += 1;
    user_config.last_completed_date = Some(today);
    user_config.current_streak = streak;
    if streak > user_config.longest_streak {
        user_config.longest_streak = streak;
    }

    if let Err(e) = config::save_config(&user_config) {
        eprintln!("Failed to save progress: {}", e);
        std::process::exit(1);
    }

    // Calculate time taken
    let submit_time = Local::now();
    let time_display = if let Some(ref init_time_str) = metadata.initialized_at {
        if let Ok(init_time) = DateTime::parse_from_rfc3339(init_time_str) {
            let duration = submit_time.signed_duration_since(init_time);
            let total_secs = duration.num_seconds();
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;

            if hours > 0 {
                format!("{}h {}m {}s", hours, minutes, seconds)
            } else if minutes > 0 {
                format!("{}m {}s", minutes, seconds)
            } else {
                format!("{}s", seconds)
            }
        } else {
            "unknown".to_string()
        }
    } else {
        "unknown".to_string()
    };

    // Display results
    println!("========================================");
    println!("  CHALLENGE COMPLETE!");
    println!("========================================");
    println!();
    println!("  Challenge:  {}", metadata.challenge_name);
    println!("  Language:   {}", metadata.language.display_name());
    println!("  Difficulty: {}", metadata.difficulty.display_name());
    println!("  Tests:      {}/{} passed", summary.passed, summary.total);
    println!("  Time taken: {}", time_display);
    println!();
    println!(
        "  Score: {} (challenge) + {} (tier) + {} (streak) = +{}",
        metadata.challenge_difficulty,
        metadata.difficulty.tier_offset(),
        streak_bonus,
        points
    );
    println!("  Streak:     {} day(s)", streak);
    println!("  BOSS Score: {}", user_config.boss_score);
    println!("  Completed:  {} challenges total", user_config.challenges_completed);
    println!();
    println!("========================================");
}

fn generic_info() {
    let mut user_config = config::load_config();
    let today = Local::now().format("%Y-%m-%d").to_string();

    if let Some(ref last_date) = user_config.last_completed_date {
        if last_date == &today {
            println!("You've completed today's challenge!");
        } else {
            println!("You still have a challenge to complete today!");
        }
    }

    println!("\nBOSS Score: {}", user_config.boss_score);
    println!("Challenges completed: {}", user_config.challenges_completed);
    println!("Current streak: {}", user_config.current_streak);
    println!("Longest streak: {}", user_config.longest_streak);
}