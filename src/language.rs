use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rs,
    Py,
    Kt,
    Java,
    C,
    Cpp,
}

impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rs => "Rust",
            Language::Py => "Python",
            Language::Kt => "Kotlin",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "C++",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Language::Rs => "rs",
            Language::Py => "py",
            Language::Kt => "kt",
            Language::Java => "java",
            Language::C => "c",
            Language::Cpp => "cpp",
        }
    }

    pub fn test_command(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Language::Rs => ("cargo", &["test"]),
            Language::Py => ("pytest", &["test_solution.py", "-v"]),
            Language::Kt => ("./gradlew", &["test"]),
            Language::Java => ("./gradlew", &["test"]),
            Language::C => ("make", &["test"]),
            Language::Cpp => ("make", &["test"]),
        }
    }
}
