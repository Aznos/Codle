mod rust;
mod python;
mod kotlin;
mod java;
mod c;
mod cpp;

use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::models::{Challenge, Difficulty, FunctionSignature, Language, RustType};

#[derive(Debug)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub output: String,
}

// --- Shared helpers ---

fn write_setup_script(output_dir: &Path, content: &str) -> Result<(), String> {
    let setup_path = output_dir.join("setup.sh");
    fs::write(&setup_path, content).map_err(|e| format!("Failed to write setup.sh: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&setup_path)
            .map_err(|e| format!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&setup_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

fn escape_for_heredoc(content: &str) -> String {
    content.to_string()
}

fn require_commands(commands: &[&str]) -> String {
    let checks: Vec<String> = commands
        .iter()
        .map(|cmd| {
            format!(
                r#"if ! command -v {cmd} &> /dev/null; then
    echo "Error: '{cmd}' is not installed. Please install it and try again."
    exit 1
fi"#,
                cmd = cmd
            )
        })
        .collect();
    checks.join("\n")
}

fn has_mut_ref_params(sig: &FunctionSignature) -> bool {
    sig.params.iter().any(|p| matches!(&p.ty, RustType::MutRef(_)))
}

fn is_void_with_mut_ref(sig: &FunctionSignature) -> bool {
    sig.return_type == RustType::Void && has_mut_ref_params(sig)
}

fn get_first_test_inputs(challenge: &Challenge) -> Option<&serde_json::Map<String, Value>> {
    challenge.tests.first().and_then(|t| t.input.as_object())
}

fn unwrap_mut_ref(ty: &RustType) -> &RustType {
    match ty {
        RustType::MutRef(inner) => inner,
        other => other,
    }
}

fn get_first_mut_ref_inner_type(sig: &FunctionSignature) -> Option<&RustType> {
    sig.params
        .iter()
        .find(|p| matches!(&p.ty, RustType::MutRef(_)))
        .map(|p| unwrap_mut_ref(&p.ty))
}

// --- Dispatch functions ---

pub fn translate_type(ty: &RustType, lang: Language) -> String {
    match lang {
        Language::Rs => rust::translate_type_rs(ty),
        Language::Py => python::translate_type_py(ty),
        Language::Kt => kotlin::translate_type_kt(ty),
        Language::Java => java::translate_type_java(ty),
        Language::C => c::translate_type_c(ty),
        Language::Cpp => cpp::translate_type_cpp(ty),
    }
}

pub fn render_value(value: &Value, ty: &RustType, lang: Language) -> String {
    match lang {
        Language::Rs => rust::render_value_rs(value, ty),
        Language::Py => python::render_value_py(value, ty),
        Language::Kt => kotlin::render_value_kt(value, ty),
        Language::Java => java::render_value_java(value, ty),
        Language::C => c::render_value_c(value, ty),
        Language::Cpp => cpp::render_value_cpp(value, ty),
    }
}

pub fn generate_scaffold(
    challenge: &Challenge,
    sig: &FunctionSignature,
    lang: Language,
    difficulty: Difficulty,
    output_dir: &Path,
) -> Result<(), String> {
    match lang {
        Language::Rs => rust::generate_rust(challenge, sig, difficulty, output_dir),
        Language::Py => python::generate_python(challenge, sig, difficulty, output_dir),
        Language::Kt => kotlin::generate_kotlin(challenge, sig, difficulty, output_dir),
        Language::Java => java::generate_java(challenge, sig, difficulty, output_dir),
        Language::C => c::generate_c(challenge, sig, difficulty, output_dir),
        Language::Cpp => cpp::generate_cpp(challenge, sig, difficulty, output_dir),
    }
}

pub fn run_tests(lang: Language) -> Result<TestSummary, String> {
    let (cmd, args) = lang.test_command();

    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run {}: {}", cmd, e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr);

    match lang {
        Language::Rs => rust::parse_rust_output(&stdout, &stderr, &combined),
        Language::Py => python::parse_pytest_output(&stdout, &stderr, &combined),
        Language::Kt | Language::Java => parse_gradle_output(&stdout, &stderr, &combined),
        Language::C | Language::Cpp => c::parse_c_output(&stdout, &stderr, &combined),
    }
}

// --- Shared output parsers ---

fn parse_gradle_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<TestSummary, String> {
    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;

    for line in combined.lines() {
        if line.contains("tests completed") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "tests" && i > 0 {
                    if let Ok(n) = parts[i - 1].parse::<usize>() {
                        total = n;
                    }
                }
                if *part == "failed" && i > 0 {
                    let num_str = parts[i - 1].trim_end_matches(',');
                    if let Ok(n) = num_str.parse::<usize>() {
                        failed = n;
                    }
                }
            }
            passed = total.saturating_sub(failed);
            break;
        }
    }

    if total == 0 {
        for line in combined.lines() {
            let trimmed = line.trim();
            if trimmed.contains("()") {
                if trimmed.ends_with("PASSED") {
                    passed += 1;
                } else if trimmed.ends_with("FAILED") {
                    failed += 1;
                }
            }
        }
        total = passed + failed;
    }

    Ok(TestSummary {
        passed,
        failed,
        total,
        output: combined.to_string(),
    })
}
