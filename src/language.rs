use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
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
}
