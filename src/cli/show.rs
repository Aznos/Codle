use crate::models::{load_daily_challenge, config};
use crate::display::display_challenge;

pub fn show_challenge() {
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
