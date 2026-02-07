use crate::models::{Difficulty, config};

pub fn handle_difficulty(level: Option<Difficulty>) {
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
