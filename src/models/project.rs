use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::difficulty::Difficulty;
use super::language::Language;

const METADATA_FILE: &str = ".codle.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub challenge_name: String,
    pub language: Language,
    pub difficulty: Difficulty,
    pub function_name: String,
    #[serde(default)]
    pub initialized_at: Option<String>,
    #[serde(default)]
    pub challenge_difficulty: u8,
}

impl ProjectMetadata {
    pub fn new(
        challenge_name: String,
        language: Language,
        difficulty: Difficulty,
        function_name: String,
        initialized_at: Option<String>,
        challenge_difficulty: u8,
    ) -> Self {
        Self {
            challenge_name,
            language,
            difficulty,
            function_name,
            initialized_at,
            challenge_difficulty,
        }
    }
}

pub fn load(dir: &Path) -> Result<ProjectMetadata, String> {
    let path = dir.join(METADATA_FILE);
    if !path.exists() {
        return Err(format!(
            "No {} found. Are you in a codle project directory?",
            METADATA_FILE
        ));
    }

    let content =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", METADATA_FILE, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {}", METADATA_FILE, e))
}

pub fn save(dir: &Path, metadata: &ProjectMetadata) -> Result<(), String> {
    let path = dir.join(METADATA_FILE);
    let content = serde_json::to_string_pretty(metadata)
        .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

    fs::write(&path, content).map_err(|e| format!("Failed to write {}: {}", METADATA_FILE, e))
}

pub fn metadata_json(metadata: &ProjectMetadata) -> String {
    serde_json::to_string_pretty(metadata).unwrap_or_default()
}
