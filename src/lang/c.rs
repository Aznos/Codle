use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    has_mut_ref_params, is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_c(ty: &RustType) -> String {
    match ty {
        RustType::I32 => "int".to_string(),
        RustType::F64 => "double".to_string(),
        RustType::Usize => "size_t".to_string(),
        RustType::Bool => "bool".to_string(),
        RustType::String => "char*".to_string(),
        RustType::Char => "char".to_string(),
        RustType::Vec(inner) => format!("{}*", translate_type_c(inner)),
        RustType::MutRef(inner) => translate_type_c(inner),
        RustType::Void => "void".to_string(),
    }
}

pub(super) fn render_value_c(value: &Value, ty: &RustType) -> String {
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
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        RustType::String => format!("\"{}\"", value.as_str().unwrap_or("")),
        RustType::Char => {
            let s = value.as_str().unwrap_or("?");
            let c = s.chars().next().unwrap_or('?');
            format!("'{}'", c)
        }
        RustType::Vec(inner) => {
            if let Some(arr) = value.as_array() {
                let items: Vec<String> = arr.iter().map(|v| render_value_c(v, inner)).collect();
                format!("{{{}}}", items.join(", "))
            } else {
                "{}".to_string()
            }
        }
        RustType::MutRef(inner) => render_value_c(value, inner),
        RustType::Void => "".to_string(),
    }
}

fn expand_c_params(sig: &FunctionSignature) -> Vec<String> {
    let mut result = Vec::new();
    for p in &sig.params {
        let inner = unwrap_mut_ref(&p.ty);
        if let RustType::Vec(elem) = inner {
            result.push(format!("{} {}[]", translate_type_c(elem), p.name));
            result.push(format!("int {}_len", p.name));
        } else {
            result.push(format!("{} {}", translate_type_c(inner), p.name));
        }
    }
    result
}

fn c_return_type(sig: &FunctionSignature) -> String {
    match &sig.return_type {
        RustType::Vec(inner) => format!("{}*", translate_type_c(inner)),
        other => translate_type_c(other),
    }
}

pub(super) fn generate_c(
    challenge: &Challenge,
    sig: &FunctionSignature,
    difficulty: Difficulty,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let params_str = expand_c_params(sig);
    let ret_type = c_return_type(sig);

    let default_return = match &sig.return_type {
        RustType::Void => String::new(),
        RustType::Bool => "    return false;\n".to_string(),
        RustType::I32 | RustType::Usize => "    return 0;\n".to_string(),
        RustType::F64 => "    return 0.0;\n".to_string(),
        RustType::String => "    return \"\";\n".to_string(),
        RustType::Vec(_) => "    return NULL;\n".to_string(),
        _ => "    return 0;\n".to_string(),
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    if let RustType::Vec(elem) = inner_ty {
                        let arr_val = render_value_c(val, inner_ty);
                        let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                        main_body.push_str(&format!(
                            "    {} {}[] = {};\n",
                            translate_type_c(elem),
                            p.name,
                            arr_val
                        ));
                        main_body.push_str(&format!("    int {}_len = {};\n", p.name, len));
                    } else {
                        main_body.push_str(&format!(
                            "    {} {} = {};\n",
                            translate_type_c(inner_ty),
                            p.name,
                            render_value_c(val, inner_ty)
                        ));
                    }
                }
            }
            let mut call_args = Vec::new();
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if matches!(inner_ty, RustType::Vec(_)) {
                    call_args.push(p.name.clone());
                    call_args.push(format!("{}_len", p.name));
                } else {
                    call_args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!("    {}({});\n", sig.name, call_args.join(", ")));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if matches!(inner_ty, RustType::Vec(_)) {
                    main_body.push_str(&format!(
                        "    for (int i = 0; i < {}_len; i++) printf(\"%d \", {}[i]);\n",
                        p.name, p.name
                    ));
                    main_body.push_str("    printf(\"\\n\");\n");
                }
            }
        } else {
            let mut call_args = Vec::new();
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    if let RustType::Vec(elem) = inner_ty {
                        let arr_val = render_value_c(val, inner_ty);
                        let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                        main_body.push_str(&format!(
                            "    {} {}[] = {};\n",
                            translate_type_c(elem),
                            p.name,
                            arr_val
                        ));
                        main_body.push_str(&format!("    int {}_len = {};\n", p.name, len));
                        call_args.push(p.name.clone());
                        call_args.push(format!("{}_len", p.name));
                    } else {
                        main_body.push_str(&format!(
                            "    {} {} = {};\n",
                            translate_type_c(inner_ty),
                            p.name,
                            render_value_c(val, inner_ty)
                        ));
                        call_args.push(p.name.clone());
                    }
                }
            }
            main_body.push_str(&format!(
                "    {} result = {}({});\n",
                ret_type,
                sig.name,
                call_args.join(", ")
            ));
            main_body.push_str("    printf(\"%d\\n\", result);\n");
        }
    }

    let includes = if sig.return_type == RustType::Void && !has_mut_ref_params(sig) {
        "#include <stdio.h>\n"
    } else {
        "#include <stdio.h>\n#include <stdbool.h>\n#include <stdlib.h>\n"
    };

    let solution_c_no_main = format!(
        r#"{includes}
{ret_type} {name}({params}) {{
{default_return}}}"#,
        includes = includes,
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
    );

    let solution_c = format!(
        r#"{includes}
{ret_type} {name}({params}) {{
{default_return}}}

int main() {{
{main_body}    return 0;
}}"#,
        includes = includes,
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
        main_body = main_body,
    );

    let tests_code = generate_c_tests(sig, &challenge.tests);

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::C,
        difficulty,
        sig.name.clone(),
        Some(chrono::Local::now().to_rfc3339()),
        challenge.difficulty,
    );
    let metadata_content = metadata_json(&metadata);

    let makefile = r#"CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g
TARGET = solution
TEST_TARGET = test_runner
SRC = solution.c
TEST_SRC = test_solution.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

test: $(TEST_TARGET)
	./$(TEST_TARGET)

$(TEST_TARGET): $(TEST_SRC) solution_lib.c
	$(CC) $(CFLAGS) -o $(TEST_TARGET) solution_lib.c $(TEST_SRC)

run: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(TARGET) $(TEST_TARGET)

.PHONY: all run clean test"#;

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

cat > Makefile << 'MAKEFILE'
{}
MAKEFILE

cat > solution.c << 'SOLUTION'
{}
SOLUTION

cat > solution_lib.c << 'SOLUTION_LIB'
{}
SOLUTION_LIB

cat > test_solution.c << 'TESTS'
{}
TESTS

cat > .codle.json << 'METADATA'
{}
METADATA

echo "Run: make && ./solution"
echo "Test: make test"
"#,
        require_commands(&["gcc", "make"]),
        makefile,
        escape_for_heredoc(&solution_c),
        escape_for_heredoc(&solution_c_no_main),
        escape_for_heredoc(&tests_code),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_c_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_code = String::new();
    test_code.push_str("#include <stdio.h>\n");
    test_code.push_str("#include <stdbool.h>\n");
    test_code.push_str("#include <stdlib.h>\n");
    test_code.push_str("#include <string.h>\n\n");

    test_code.push_str("// Forward declaration - implemented in solution.c\n");

    test_code.push_str("\nint main() {\n");
    test_code.push_str("    int passed = 0, failed = 0;\n\n");

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;

        if let Some(inputs) = test.input.as_object() {
            test_code.push_str(&format!("    // Test {}\n", test_num));
            test_code.push_str("    {\n");

            let mut call_args = Vec::new();

            if is_void_with_mut_ref(sig) {
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        if let RustType::Vec(elem) = inner_ty {
                            let arr_val = super::render_value(val, inner_ty, Language::C);
                            let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                            test_code.push_str(&format!(
                                "        {} {}_arr[] = {};\n",
                                super::translate_type(elem, Language::C),
                                p.name,
                                arr_val
                            ));
                            test_code.push_str(&format!("        int {}_len = {};\n", p.name, len));
                            call_args.push(format!("{}_arr", p.name));
                            call_args.push(format!("{}_len", p.name));
                        } else {
                            test_code.push_str(&format!(
                                "        {} {} = {};\n",
                                super::translate_type(inner_ty, Language::C),
                                p.name,
                                super::render_value(val, inner_ty, Language::C)
                            ));
                            call_args.push(p.name.clone());
                        }
                    }
                }
                test_code.push_str(&format!(
                    "        {}({});\n",
                    sig.name,
                    call_args.join(", ")
                ));

                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    if let RustType::Vec(_) = inner {
                        if let Some(expected_arr) = test.expected.as_array() {
                            test_code.push_str("        int test_passed = 1;\n");
                            for (j, expected_val) in expected_arr.iter().enumerate() {
                                test_code.push_str(&format!(
                                    "        if ({}_arr[{}] != {}) test_passed = 0;\n",
                                    p.name,
                                    j,
                                    super::render_value(expected_val, &RustType::I32, Language::C)
                                ));
                            }
                            test_code.push_str(&format!(
                                "        if (test_passed) {{ printf(\"Test {}: PASS\\n\"); passed++; }}\n",
                                test_num
                            ));
                            test_code.push_str(&format!(
                                "        else {{ printf(\"Test {}: FAIL\\n\"); failed++; }}\n",
                                test_num
                            ));
                        }
                    }
                }
            } else {
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        if let RustType::Vec(elem) = inner_ty {
                            let arr_val = super::render_value(val, inner_ty, Language::C);
                            let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                            test_code.push_str(&format!(
                                "        {} {}_arr[] = {};\n",
                                super::translate_type(elem, Language::C),
                                p.name,
                                arr_val
                            ));
                            test_code.push_str(&format!("        int {}_len = {};\n", p.name, len));
                            call_args.push(format!("{}_arr", p.name));
                            call_args.push(format!("{}_len", p.name));
                        } else {
                            test_code.push_str(&format!(
                                "        {} {} = {};\n",
                                super::translate_type(inner_ty, Language::C),
                                p.name,
                                super::render_value(val, inner_ty, Language::C)
                            ));
                            call_args.push(p.name.clone());
                        }
                    }
                }

                match &sig.return_type {
                    RustType::Vec(inner) => {
                        test_code.push_str(&format!(
                            "        {}* result = {}({});\n",
                            super::translate_type(inner, Language::C),
                            sig.name,
                            call_args.join(", ")
                        ));
                        if let Some(expected_arr) = test.expected.as_array() {
                            test_code.push_str("        int test_passed = 1;\n");
                            for (j, expected_val) in expected_arr.iter().enumerate() {
                                test_code.push_str(&format!(
                                    "        if (result[{}] != {}) test_passed = 0;\n",
                                    j,
                                    super::render_value(expected_val, inner, Language::C)
                                ));
                            }
                            test_code.push_str(&format!(
                                "        if (test_passed) {{ printf(\"Test {}: PASS\\n\"); passed++; }}\n",
                                test_num
                            ));
                            test_code.push_str(&format!(
                                "        else {{ printf(\"Test {}: FAIL\\n\"); failed++; }}\n",
                                test_num
                            ));
                        }
                    }
                    _ => {
                        test_code.push_str(&format!(
                            "        {} result = {}({});\n",
                            super::translate_type(&sig.return_type, Language::C),
                            sig.name,
                            call_args.join(", ")
                        ));
                        let expected = super::render_value(&test.expected, &sig.return_type, Language::C);
                        test_code.push_str(&format!(
                            "        if (result == {}) {{ printf(\"Test {}: PASS\\n\"); passed++; }}\n",
                            expected, test_num
                        ));
                        test_code.push_str(&format!(
                            "        else {{ printf(\"Test {}: FAIL (expected {}, got %d)\\n\", result); failed++; }}\n",
                            test_num, expected
                        ));
                    }
                }
            }

            test_code.push_str("    }\n\n");
        }
    }

    test_code.push_str("    printf(\"\\n%d/%d tests passed\\n\", passed, passed + failed);\n");
    test_code.push_str("    return failed > 0 ? 1 : 0;\n");
    test_code.push_str("}\n");

    test_code
}

pub(super) fn parse_c_output(_stdout: &str, _stderr: &str, combined: &str) -> Result<super::TestSummary, String> {
    let mut passed = 0;
    let mut failed = 0;

    for line in combined.lines() {
        if line.contains("tests passed") {
            let parts: Vec<&str> = line.split('/').collect();
            if parts.len() >= 2 {
                if let Ok(p) = parts[0].trim().parse::<usize>() {
                    passed = p;
                }
                let after_slash = parts[1].split_whitespace().next().unwrap_or("0");
                if let Ok(t) = after_slash.parse::<usize>() {
                    failed = t.saturating_sub(passed);
                }
            }
            break;
        }
    }

    if passed == 0 && failed == 0 {
        for line in combined.lines() {
            if line.contains(": PASS") {
                passed += 1;
            } else if line.contains(": FAIL") {
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
