use std::process::Command;

use crate::language::Language;

#[derive(Debug)]
pub struct TestSummary {
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub output: String,
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

    parse_output(lang, &stdout, &stderr, &combined)
}

fn parse_output(
    lang: Language,
    stdout: &str,
    stderr: &str,
    combined: &str,
) -> Result<TestSummary, String> {
    match lang {
        Language::Rs => parse_rust_output(stdout, stderr, combined),
        Language::Py => parse_pytest_output(stdout, stderr, combined),
        Language::Kt | Language::Java => parse_gradle_output(stdout, stderr, combined),
        Language::C | Language::Cpp => parse_c_output(stdout, stderr, combined),
    }
}

fn parse_rust_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<TestSummary, String> {
    // Cargo test output format:
    // "test result: ok. X passed; Y failed; Z ignored; ..."
    // Or individual: "test tests::test_1 ... ok"

    let mut passed = 0;
    let mut failed = 0;

    // Look for the summary line
    for line in combined.lines() {
        if line.starts_with("test result:") {
            // Parse "test result: ok. 3 passed; 0 failed; ..."
            if let Some(passed_part) = line.split(';').next() {
                if let Some(num_str) = passed_part.split_whitespace().find(|s| s.parse::<usize>().is_ok()) {
                    passed = num_str.parse().unwrap_or(0);
                }
            }
            for part in line.split(';') {
                if part.contains("failed") {
                    if let Some(num_str) = part.split_whitespace().find(|s| s.parse::<usize>().is_ok()) {
                        failed = num_str.parse().unwrap_or(0);
                    }
                }
            }
            break;
        }
    }

    // Fallback: count individual test lines
    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains(" ... ok") {
                passed += 1;
            } else if line.contains(" ... FAILED") {
                failed += 1;
            }
        }
    }

    Ok(TestSummary {
        passed,
        failed,
        total: passed + failed,
        output: combined.to_string(),
    })
}

fn parse_pytest_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<TestSummary, String> {
    // Pytest output format:
    // "===== X passed, Y failed in 0.00s ====="
    // Or: "===== X passed in 0.00s ====="

    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        // Look for summary line with "passed" or "failed"
        if line.contains("passed") || line.contains("failed") {
            // Try to find "N passed"
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "passed" && i > 0 {
                    if let Ok(n) = parts[i - 1].parse::<usize>() {
                        passed = n;
                    }
                }
                if *part == "failed" && i > 0 {
                    if let Ok(n) = parts[i - 1].parse::<usize>() {
                        failed = n;
                    }
                }
            }
        }
    }

    // Fallback: count PASSED/FAILED markers
    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains("PASSED") {
                passed += 1;
            } else if line.contains("FAILED") {
                failed += 1;
            }
        }
    }

    Ok(TestSummary {
        passed,
        failed,
        total: passed + failed,
        output: combined.to_string(),
    })
}

fn parse_gradle_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<TestSummary, String> {
    // Gradle test output:
    // "X tests completed, Y failed"
    // Individual: "AppTest > test1() PASSED" or "AppTest > test1() FAILED"

    let mut passed = 0;
    let mut failed = 0;
    let mut total = 0;

    for line in combined.lines() {
        // Look for "X tests completed, Y failed"
        if line.contains("tests completed") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "tests" && i > 0 {
                    if let Ok(n) = parts[i - 1].parse::<usize>() {
                        total = n;
                    }
                }
                if *part == "failed" && i > 0 {
                    // Remove trailing comma if present
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

    // Fallback: count individual test result lines
    // Match "TestClass > testMethod() PASSED/FAILED" pattern, not build status lines
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

fn parse_c_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<TestSummary, String> {
    // Our C/C++ test harness outputs:
    // "Test N: PASS" or "Test N: FAIL"
    // "X/Y tests passed"

    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        if line.contains("tests passed") {
            // Parse "X/Y tests passed"
            let parts: Vec<&str> = line.split('/').collect();
            if parts.len() >= 2 {
                if let Ok(p) = parts[0].trim().parse::<usize>() {
                    passed = p;
                }
                // Total is after the slash, before "tests"
                let after_slash = parts[1].split_whitespace().next().unwrap_or("0");
                if let Ok(t) = after_slash.parse::<usize>() {
                    failed = t.saturating_sub(passed);
                }
            }
            break;
        }
    }

    // Fallback: count individual test lines
    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains(": PASS") {
                passed += 1;
            } else if line.contains(": FAIL") {
                failed += 1;
            }
        }
    }

    Ok(TestSummary {
        passed,
        failed,
        total: passed + failed,
        output: combined.to_string(),
    })
}
