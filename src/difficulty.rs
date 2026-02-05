use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Extreme,
}

impl Difficulty {
    pub fn as_str(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medium => "medium",
            Difficulty::Hard => "hard",
            Difficulty::Extreme => "extreme",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Extreme => "Extreme",
        }
    }

    pub fn boss_multiplier(&self) -> f32 {
        match self {
            Difficulty::Easy => 1.0,
            Difficulty::Medium => 1.5,
            Difficulty::Hard => 2.5,
            Difficulty::Extreme => 4.0,
        }
    }

    pub fn points_for_completion(&self) -> u32 {
        (100.0 * self.boss_multiplier()) as u32
    }
}