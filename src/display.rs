use crate::models::Challenge;

pub fn display_challenge(challenge: &Challenge) {
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