use std::env;

use chrono::{DateTime, Local};

use crate::models::{calculate_boss_score, config, project};
use crate::lang::run_tests;

pub fn submit_solution() {
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
