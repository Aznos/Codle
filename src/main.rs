use std::fs;
use std::path::PathBuf;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme
}

impl Difficulty {
    fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
            Difficulty::Extreme => "extreme",
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Challenge {
    pub name: String,
    pub difficulty: u8,
    pub short_description: String,
    pub description: String,
    pub function_signature: String,
    pub tests: Vec<TestCase>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestCase {
    pub input: Value,
    pub expected: Value,
}

fn get_challenges_dir() -> PathBuf {
    let exe_path = std::env::current_exe().unwrap_or_default();
    let mut path = exe_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();

    let possible_paths = vec![
        PathBuf::from("challenges"),
        path.join("challenges"),
        {
            path.pop();
            path.pop();
            path.join("challenges")
        }
    ];

    for p in possible_paths {
        if p.exists() {
            return p;
        }
    }

    PathBuf::from("challenges")
}

fn load_random_challenge(difficulty: Difficulty) -> Result<Challenge, String> {
    let challenges_dir = get_challenges_dir();
    let difficulty_dir = challenges_dir.join(difficulty.as_str());

    if !difficulty_dir.exists() {
        return Err(format!(
            "Challenges directory not found: {}",
            difficulty_dir.display()
        ));
    }

    let entries: Vec<_> = fs::read_dir(&difficulty_dir)
        .map_err(|e| format!("Failed to read dir: {}", e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        return Err(format!(
            "No challenges found in {} difficulty",
            difficulty.as_str()
        ));
    }

    let mut rng = rand::thread_rng();
    let chosen = entries.choose(&mut rng).unwrap();

    let content = fs::read_to_string(chosen.path())
        .map_err(|e| format!("Failed to read challenges: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to deserialize challenges: {}", e))
}

fn display_challenge(challenge: &Challenge) {
    println!("\n{}", "=".repeat(60));
    println!("  CODLE - Daily Coding Challenge");
    println!("{}\n", "=".repeat(60));

    println!("Challenge: {}", challenge.name);
    println!("Difficulty: {}/10", challenge.difficulty);
    println!("Summary: {}\n", challenge.short_description);

    println!("{}", "-".repeat(60));
    println!("{}", challenge.description);
    println!("{}", "-".repeat(60));

    println!("\nFunction Signature:");
    println!("  {}\n", challenge.function_signature);

    println!("Test Cases: {} total", challenge.tests.len());

    println!("\n{}", "=".repeat(60));
    println!("Run `codle test` to check your solution");
    println!("Run `codle submit` when you're ready to submit");
    println!("{}\n", "=".repeat(60));
}

fn main() {
    let difficulty = Difficulty::Easy;

    match load_random_challenge(difficulty) {
        Ok(challenge) => {
            display_challenge(&challenge);
        }
        Err(e) => {
            eprintln!("Failed to load challenges: {}", e);
            std::process::exit(1);
        }
    }
}