mod difficulty;
mod challenge;
mod display;

use crate::challenge::{load_random_challenge};
use crate::difficulty::Difficulty;
use crate::display::display_challenge;

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