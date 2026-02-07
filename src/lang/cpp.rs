use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_cpp(ty: &RustType) -> String {
    match ty {
        RustType::I32 => "int".to_string(),
        RustType::F64 => "double".to_string(),
        RustType::Usize => "size_t".to_string(),
        RustType::Bool => "bool".to_string(),
        RustType::String => "std::string".to_string(),
        RustType::Char => "char".to_string(),
        RustType::Vec(inner) => format!("std::vector<{}>", translate_type_cpp(inner)),
        RustType::MutRef(inner) => format!("{}&", translate_type_cpp(inner)),
        RustType::Void => "void".to_string(),
    }
}

pub(super) fn render_value_cpp(value: &Value, ty: &RustType) -> String {
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
        RustType::String => format!("\"{}\"", value.as_str().unwrap_or("")),
        RustType::Char => {
            let s = value.as_str().unwrap_or("?");
            let c = s.chars().next().unwrap_or('?');
            format!("'{}'", c)
        }
        RustType::Vec(inner) => {
            if let Some(arr) = value.as_array() {
                let items: Vec<String> = arr.iter().map(|v| render_value_cpp(v, inner)).collect();
                format!("{{{}}}", items.join(", "))
            } else {
                "{}".to_string()
            }
        }
        RustType::MutRef(inner) => render_value_cpp(value, inner),
        RustType::Void => "".to_string(),
    }
}

pub(super) fn generate_cpp(
    challenge: &Challenge,
    sig: &FunctionSignature,
    difficulty: Difficulty,
    output_dir: &std::path::Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{} {}", super::translate_type(&p.ty, Language::Cpp), p.name))
        .collect();

    let ret_type = super::translate_type(&sig.return_type, Language::Cpp);

    let default_return = match &sig.return_type {
        RustType::Void => String::new(),
        RustType::Bool => "    return false;\n".to_string(),
        RustType::I32 | RustType::Usize => "    return 0;\n".to_string(),
        RustType::F64 => "    return 0.0;\n".to_string(),
        RustType::String => "    return \"\";\n".to_string(),
        RustType::Vec(_) => "    return {};\n".to_string(),
        _ => "    return {};\n".to_string(),
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} {} = {};\n",
                        super::translate_type(inner_ty, Language::Cpp),
                        p.name,
                        super::render_value(val, inner_ty, Language::Cpp)
                    ));
                }
            }
            let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
            main_body.push_str(&format!("    {}({});\n", sig.name, call_args.join(", ")));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let RustType::Vec(_) = inner_ty {
                    main_body.push_str(&format!(
                        "    for (const auto& x : {}) std::cout << x << \" \";\n",
                        p.name
                    ));
                    main_body.push_str("    std::cout << std::endl;\n");
                } else {
                    main_body.push_str(&format!("    std::cout << {} << std::endl;\n", p.name));
                }
            }
        } else {
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} {} = {};\n",
                        super::translate_type(unwrap_mut_ref(&p.ty), Language::Cpp),
                        p.name,
                        super::render_value(val, unwrap_mut_ref(&p.ty), Language::Cpp)
                    ));
                    args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!(
                "    auto result = {}({});\n",
                sig.name,
                args.join(", ")
            ));
            if matches!(&sig.return_type, RustType::Vec(_)) {
                main_body
                    .push_str("    for (const auto& x : result) std::cout << x << \" \";\n");
                main_body.push_str("    std::cout << std::endl;\n");
            } else {
                main_body.push_str("    std::cout << result << std::endl;\n");
            }
        }
    }

    let mut includes = vec!["#include <iostream>"];
    let needs_vector = sig.params.iter().any(|p| {
        matches!(unwrap_mut_ref(&p.ty), RustType::Vec(_))
    }) || matches!(&sig.return_type, RustType::Vec(_));
    let needs_string = sig.params.iter().any(|p| {
        matches!(unwrap_mut_ref(&p.ty), RustType::String)
    }) || matches!(&sig.return_type, RustType::String);

    if needs_vector {
        includes.push("#include <vector>");
    }
    if needs_string {
        includes.push("#include <string>");
    }

    let solution_hpp = format!(
        r#"#pragma once
{}
{}

{} {}({});"#,
        if needs_vector { "#include <vector>" } else { "" },
        if needs_string { "#include <string>" } else { "" },
        ret_type,
        sig.name,
        params_str.join(", ")
    );

    let solution_cpp_lib = format!(
        r#"{includes}
#include "solution.hpp"

{ret_type} {name}({params}) {{
{default_return}}}"#,
        includes = includes.join("\n"),
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
    );

    let solution_cpp = format!(
        r#"{includes}

{ret_type} {name}({params}) {{
{default_return}}}

int main() {{
{main_body}    return 0;
}}"#,
        includes = includes.join("\n"),
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
        main_body = main_body,
    );

    let tests_code = generate_cpp_tests(sig, &challenge.tests);

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::Cpp,
        difficulty,
        sig.name.clone(),
        Some(chrono::Local::now().to_rfc3339()),
        challenge.difficulty,
    );
    let metadata_content = metadata_json(&metadata);

    let makefile = r#"CXX = g++
CXXFLAGS = -Wall -Wextra -std=c++17 -g
TARGET = solution
TEST_TARGET = test_runner
SRC = solution.cpp
TEST_SRC = test_solution.cpp
LIB_SRC = solution_lib.cpp

all: $(TARGET)

$(TARGET): $(SRC)
	$(CXX) $(CXXFLAGS) -o $(TARGET) $(SRC)

test: $(TEST_TARGET)
	./$(TEST_TARGET)

$(TEST_TARGET): $(TEST_SRC) $(LIB_SRC)
	$(CXX) $(CXXFLAGS) -o $(TEST_TARGET) $(LIB_SRC) $(TEST_SRC)

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

cat > solution.cpp << 'SOLUTION'
{}
SOLUTION

cat > solution.hpp << 'HEADER'
{}
HEADER

cat > solution_lib.cpp << 'SOLUTION_LIB'
{}
SOLUTION_LIB

cat > test_solution.cpp << 'TESTS'
{}
TESTS

cat > .codle.json << 'METADATA'
{}
METADATA

echo "Run: make && ./solution"
echo "Test: make test"
"#,
        require_commands(&["g++", "make"]),
        makefile,
        escape_for_heredoc(&solution_cpp),
        escape_for_heredoc(&solution_hpp),
        escape_for_heredoc(&solution_cpp_lib),
        escape_for_heredoc(&tests_code),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_cpp_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_code = String::new();
    test_code.push_str("#include <iostream>\n");
    test_code.push_str("#include <vector>\n");
    test_code.push_str("#include <string>\n\n");

    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{} {}", super::translate_type(&p.ty, Language::Cpp), p.name))
        .collect();
    let ret_type = super::translate_type(&sig.return_type, Language::Cpp);
    test_code.push_str(&format!(
        "// Forward declaration - implemented in solution.cpp\n{} {}({});\n\n",
        ret_type,
        sig.name,
        params_str.join(", ")
    ));

    test_code.push_str("int main() {\n");
    test_code.push_str("    int passed = 0, failed = 0;\n\n");

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;

        if let Some(inputs) = test.input.as_object() {
            test_code.push_str(&format!("    // Test {}\n", test_num));
            test_code.push_str("    {\n");

            if is_void_with_mut_ref(sig) {
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        test_code.push_str(&format!(
                            "        {} {} = {};\n",
                            super::translate_type(inner_ty, Language::Cpp),
                            p.name,
                            super::render_value(val, inner_ty, Language::Cpp)
                        ));
                    }
                }
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
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
                    let expected = super::render_value(&test.expected, inner, Language::Cpp);
                    test_code.push_str(&format!(
                        "        if ({} == {}) {{ std::cout << \"Test {}: PASS\" << std::endl; passed++; }}\n",
                        p.name, expected, test_num
                    ));
                    test_code.push_str(&format!(
                        "        else {{ std::cout << \"Test {}: FAIL\" << std::endl; failed++; }}\n",
                        test_num
                    ));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        test_code.push_str(&format!(
                            "        {} {} = {};\n",
                            super::translate_type(unwrap_mut_ref(&p.ty), Language::Cpp),
                            p.name,
                            super::render_value(val, unwrap_mut_ref(&p.ty), Language::Cpp)
                        ));
                        args.push(p.name.clone());
                    }
                }
                test_code.push_str(&format!(
                    "        auto result = {}({});\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = super::render_value(&test.expected, &sig.return_type, Language::Cpp);
                test_code.push_str(&format!(
                    "        if (result == {}) {{ std::cout << \"Test {}: PASS\" << std::endl; passed++; }}\n",
                    expected, test_num
                ));
                test_code.push_str(&format!(
                    "        else {{ std::cout << \"Test {}: FAIL\" << std::endl; failed++; }}\n",
                    test_num
                ));
            }

            test_code.push_str("    }\n\n");
        }
    }

    test_code.push_str("    std::cout << std::endl << passed << \"/\" << (passed + failed) << \" tests passed\" << std::endl;\n");
    test_code.push_str("    return failed > 0 ? 1 : 0;\n");
    test_code.push_str("}\n");

    test_code
}
