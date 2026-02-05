use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::difficulty::Difficulty;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub difficulty: Difficulty,
    pub boss_score: u32,
    pub challenges_completed: u32,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Medium,
            boss_score: 0,
            challenges_completed: 0,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".config").join("codle").join("config.json")
}

pub fn load_config() -> UserConfig {
    let path = get_config_path();
    if !path.exists() {
        return UserConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => UserConfig::default(),
    }
}

pub fn save_config(config: &UserConfig) -> Result<(), std::io::Error> {
    let path = get_config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let contents = serde_json::to_string_pretty(config)?;
    fs::write(path, contents)
}
