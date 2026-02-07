mod show;
mod init;
mod difficulty_cmd;
mod test;
mod submit;

use chrono::Local;
use clap::{Parser, Subcommand};

use crate::models::{Difficulty, Language, config};

#[derive(Parser)]
#[command(name = "codle")]
#[command(about = "Daily coding challenges", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    Init {
        #[arg(value_enum)]
        language: Language,
    },
    Difficulty {
        #[arg(value_enum)]
        level: Option<Difficulty>,
    },
    Test,
    Submit,
    Info,
}

pub fn run(cli: Cli) {
    match cli.command {
        None => show::show_challenge(),
        Some(Commands::Init { language }) => init::init_challenge(language),
        Some(Commands::Difficulty { level }) => difficulty_cmd::handle_difficulty(level),
        Some(Commands::Test) => test::test_solution(),
        Some(Commands::Submit) => submit::submit_solution(),
        Some(Commands::Info) => generic_info(),
    }
}

fn generic_info() {
    let user_config = config::load_config();
    let today = Local::now().format("%Y-%m-%d").to_string();

    if let Some(ref last_date) = user_config.last_completed_date {
        if last_date == &today {
            println!("You've completed today's challenge!");
        } else {
            println!("You still have a challenge to complete today!");
        }
    }

    println!("\nBOSS Score: {}", user_config.boss_score);
    println!("Challenges completed: {}", user_config.challenges_completed);
    println!("Current streak: {}", user_config.current_streak);
    println!("Longest streak: {}", user_config.longest_streak);
}
