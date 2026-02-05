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

    pub fn tier_offset(&self) -> u32 {
        match self {
            Difficulty::Easy => 0,
            Difficulty::Medium => 1,
            Difficulty::Hard => 2,
            Difficulty::Extreme => 3,
        }
    }
}

pub fn calculate_boss_score(challenge_difficulty: u8, tier: &Difficulty, streak: u32) -> u32 {
    let base = challenge_difficulty as u32;
    let tier_bonus = tier.tier_offset();
    let streak_bonus = streak.min(5);
    base + tier_bonus + streak_bonus
}