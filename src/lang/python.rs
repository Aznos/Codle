use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_py(ty: &RustType) -> String {
    match ty {
        RustType::I32 | RustType::Usize => "int".to_string(),
        RustType::F64 => "float".to_string(),
        RustType::Bool => "bool".to_string(),
        RustType::String => "str".to_string(),
        RustType::Char => "str".to_string(),
        RustType::Vec(inner) => format!("list[{}]", translate_type_py(inner)),
        RustType::MutRef(inner) => translate_type_py(inner),
        RustType::Void => "None".to_string(),
    }
}

pub(super) fn render_value_py(value: &Value, ty: &RustType) -> String {
    match ty {
        RustType::I32 | RustType::Usize => format!("{}", value.as_i64().unwrap_or(0)),
        RustType::F64 => {
            let n = value.as_f64().unwrap_or(0.0);
            if n.fract() == 0.0 {
                format!("{:.1}", n)
            } else {
                format!("{}", n)
            }
        }
        RustType::Bool => {
            if value.as_bool().unwrap_or(false) {
                "True".to_string()
            } else {
                "False".to_string()
            }
        }
        RustType::String => format!("\"{}\"", value.as_str().unwrap_or("")),
        RustType::Char => format!("\"{}\"", value.as_str().unwrap_or("?")),
        RustType::Vec(inner) => {
            if let Some(arr) = value.as_array() {
                let items: Vec<String> = arr.iter().map(|v| render_value_py(v, inner)).collect();
                format!("[{}]", items.join(", "))
            } else {
                "[]".to_string()
            }
        }
        RustType::MutRef(inner) => render_value_py(value, inner),
        RustType::Void => "None".to_string(),
    }
}

pub(super) fn generate_python(
    challenge: &Challenge,
    sig: &FunctionSignature,
    difficulty: Difficulty,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                super::translate_type(unwrap_mut_ref(&p.ty), Language::Py)
            )
        })
        .collect();
    let ret_hint = if sig.return_type == RustType::Void {
        " -> None".to_string()
    } else {
        format!(" -> {}", super::translate_type(&sig.return_type, Language::Py))
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} = {}\n",
                        p.name,
                        super::render_value(val, inner_ty, Language::Py)
                    ));
                }
            }
            let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
            main_body.push_str(&format!("    {}({})\n", sig.name, call_args.join(", ")));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                main_body.push_str(&format!("    print({})\n", p.name));
            }
        } else {
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} = {}\n",
                        p.name,
                        super::render_value(val, unwrap_mut_ref(&p.ty), Language::Py)
                    ));
                    args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!(
                "    result = {}({})\n",
                sig.name,
                args.join(", ")
            ));
            main_body.push_str("    print(result)\n");
        }
    }

    let solution_py = format!(
        r#"def {}({}){}:
    pass


if __name__ == "__main__":
{}"#,
        sig.name,
        params_str.join(", "),
        ret_hint,
        main_body,
    );

    let tests_code = generate_python_tests(sig, &challenge.tests);

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::Py,
        difficulty,
        sig.name.clone(),
        Some(chrono::Local::now().to_rfc3339()),
        challenge.difficulty,
    );
    let metadata_content = metadata_json(&metadata);

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

python3 -m venv venv
source venv/bin/activate

cat > requirements.txt << 'EOF'
pytest
EOF

pip install -r requirements.txt

cat > solution.py << 'SOLUTION'
{}
SOLUTION

cat > test_solution.py << 'TESTS'
{}
TESTS

cat > .codle.json << 'METADATA'
{}
METADATA

echo "Run: source venv/bin/activate && python solution.py"
echo "Test: source venv/bin/activate && pytest test_solution.py -v"
"#,
        require_commands(&["python3", "pip"]),
        escape_for_heredoc(&solution_py),
        escape_for_heredoc(&tests_code),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_python_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_fns = Vec::new();
    test_fns.push(format!("from solution import {}\n", sig.name));

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;
        let mut body = String::new();

        if let Some(inputs) = test.input.as_object() {
            if is_void_with_mut_ref(sig) {
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "    {} = {}\n",
                            p.name,
                            super::render_value(val, inner_ty, Language::Py)
                        ));
                    }
                }
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
                body.push_str(&format!("    {}({})\n", sig.name, call_args.join(", ")));
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = super::render_value(&test.expected, inner, Language::Py);
                    body.push_str(&format!("    assert {} == {}\n", p.name, expected));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "    {} = {}\n",
                            p.name,
                            super::render_value(val, unwrap_mut_ref(&p.ty), Language::Py)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "    result = {}({})\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = super::render_value(&test.expected, &sig.return_type, Language::Py);
                body.push_str(&format!("    assert result == {}\n", expected));
            }
        }

        test_fns.push(format!(
            r#"
def test_{}():
{}"#,
            test_num, body
        ));
    }

    test_fns.join("\n")
}

pub(super) fn parse_pytest_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<super::TestSummary, String> {
    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        if line.contains("passed") || line.contains("failed") {
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

    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains("PASSED") {
                passed += 1;
            } else if line.contains("FAILED") {
                failed += 1;
            }
        }
    }

    Ok(super::TestSummary {
        passed,
        failed,
        total: passed + failed,
        output: combined.to_string(),
    })
}
