use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_rs(ty: &RustType) -> String {
    match ty {
        RustType::I32 => "i32".to_string(),
        RustType::F64 => "f64".to_string(),
        RustType::Usize => "usize".to_string(),
        RustType::Bool => "bool".to_string(),
        RustType::String => "String".to_string(),
        RustType::Char => "char".to_string(),
        RustType::Vec(inner) => format!("Vec<{}>", translate_type_rs(inner)),
        RustType::MutRef(inner) => format!("&mut {}", translate_type_rs(inner)),
        RustType::Void => "()".to_string(),
    }
}

pub(super) fn render_value_rs(value: &Value, ty: &RustType) -> String {
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
        RustType::Bool => format!("{}", value.as_bool().unwrap_or(false)),
        RustType::String => format!("\"{}\".to_string()", value.as_str().unwrap_or("")),
        RustType::Char => {
            let s = value.as_str().unwrap_or("?");
            let c = s.chars().next().unwrap_or('?');
            format!("'{}'", c)
        }
        RustType::Vec(inner) => {
            if let Some(arr) = value.as_array() {
                let items: Vec<String> = arr.iter().map(|v| render_value_rs(v, inner)).collect();
                format!("vec![{}]", items.join(", "))
            } else {
                "vec![]".to_string()
            }
        }
        RustType::MutRef(inner) => render_value_rs(value, inner),
        RustType::Void => "()".to_string(),
    }
}

pub(super) fn generate_rust(
    challenge: &Challenge,
    sig: &FunctionSignature,
    difficulty: Difficulty,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, super::translate_type(&p.ty, Language::Rs)))
        .collect();
    let ret_str = if sig.return_type == RustType::Void {
        String::new()
    } else {
        format!(" -> {}", super::translate_type(&sig.return_type, Language::Rs))
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                if let RustType::MutRef(inner) = &p.ty {
                    if let Some(val) = inputs.get(&p.name) {
                        main_body.push_str(&format!(
                            "    let mut {} = {};\n",
                            p.name,
                            super::render_value(val, inner, Language::Rs)
                        ));
                    }
                } else if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    let {} = {};\n",
                        p.name,
                        super::render_value(val, &p.ty, Language::Rs)
                    ));
                }
            }
            let call_args: Vec<String> = sig
                .params
                .iter()
                .map(|p| {
                    if matches!(&p.ty, RustType::MutRef(_)) {
                        format!("&mut {}", p.name)
                    } else {
                        p.name.clone()
                    }
                })
                .collect();
            main_body.push_str(&format!("    {}({});\n", sig.name, call_args.join(", ")));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                main_body.push_str(&format!("    println!(\"{{:?}}\", {});\n", p.name));
            }
        } else {
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    let {} = {};\n",
                        p.name,
                        super::render_value(val, &p.ty, Language::Rs)
                    ));
                    args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!(
                "    let result = {}({});\n",
                sig.name,
                args.join(", ")
            ));
            main_body.push_str("    println!(\"{:?}\", result);\n");
        }
    }

    let tests_code = generate_rust_tests(sig, &challenge.tests);

    let main_rs = format!(
        r#"fn {}({}){} {{
    todo!()
}}

fn main() {{
{}}}
{}"#,
        sig.name,
        params_str.join(", "),
        ret_str,
        main_body,
        tests_code
    );

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::Rs,
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

cargo init --name "{}"

cat > src/main.rs << 'SOLUTION'
{}
SOLUTION

cat > .codle.json << 'METADATA'
{}
METADATA

echo "Run: cargo run"
echo "Test: cargo test"
"#,
        require_commands(&["cargo"]),
        sig.name,
        escape_for_heredoc(&main_rs),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_rust_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_fns = Vec::new();

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;
        let mut body = String::new();

        if let Some(inputs) = test.input.as_object() {
            if is_void_with_mut_ref(sig) {
                for p in &sig.params {
                    if let RustType::MutRef(inner) = &p.ty {
                        if let Some(val) = inputs.get(&p.name) {
                            body.push_str(&format!(
                                "        let mut {} = {};\n",
                                p.name,
                                super::render_value(val, inner, Language::Rs)
                            ));
                        }
                    } else if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        let {} = {};\n",
                            p.name,
                            super::render_value(val, &p.ty, Language::Rs)
                        ));
                    }
                }
                let call_args: Vec<String> = sig
                    .params
                    .iter()
                    .map(|p| {
                        if matches!(&p.ty, RustType::MutRef(_)) {
                            format!("&mut {}", p.name)
                        } else {
                            p.name.clone()
                        }
                    })
                    .collect();
                body.push_str(&format!("        {}({});\n", sig.name, call_args.join(", ")));
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = super::render_value(&test.expected, inner, Language::Rs);
                    body.push_str(&format!("        assert_eq!({}, {});\n", p.name, expected));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        let {} = {};\n",
                            p.name,
                            super::render_value(val, &p.ty, Language::Rs)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        let result = {}({});\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = super::render_value(&test.expected, &sig.return_type, Language::Rs);
                body.push_str(&format!("        assert_eq!(result, {});\n", expected));
            }
        }

        test_fns.push(format!(
            r#"    #[test]
    fn test_{}() {{
{}    }}"#,
            test_num, body
        ));
    }

    format!(
        r#"
#[cfg(test)]
mod tests {{
    use super::*;

{}
}}"#,
        test_fns.join("\n\n")
    )
}

pub(super) fn parse_rust_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<super::TestSummary, String> {
    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        if line.starts_with("test result:") {
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

    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains(" ... ok") {
                passed += 1;
            } else if line.contains(" ... FAILED") {
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
