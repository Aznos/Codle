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

fn main() {
    println!("Hello world!");
}