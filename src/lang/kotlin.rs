use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_kt(ty: &RustType) -> String {
    match ty {
        RustType::I32 | RustType::Usize => "Int".to_string(),
        RustType::F64 => "Double".to_string(),
        RustType::Bool => "Boolean".to_string(),
        RustType::String => "String".to_string(),
        RustType::Char => "Char".to_string(),
        RustType::Vec(inner) => format!("MutableList<{}>", translate_type_kt(inner)),
        RustType::MutRef(inner) => translate_type_kt(inner),
        RustType::Void => "Unit".to_string(),
    }
}

pub(super) fn render_value_kt(value: &Value, ty: &RustType) -> String {
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
                let items: Vec<String> = arr.iter().map(|v| render_value_kt(v, inner)).collect();
                format!("mutableListOf({})", items.join(", "))
            } else {
                "mutableListOf()".to_string()
            }
        }
        RustType::MutRef(inner) => render_value_kt(value, inner),
        RustType::Void => "Unit".to_string(),
    }
}

pub(super) fn generate_kotlin(
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
                super::translate_type(unwrap_mut_ref(&p.ty), Language::Kt)
            )
        })
        .collect();
    let ret_str = if sig.return_type == RustType::Void {
        String::new()
    } else {
        format!(": {}", super::translate_type(&sig.return_type, Language::Kt))
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    val {} = {}\n",
                        p.name,
                        super::render_value(val, inner_ty, Language::Kt)
                    ));
                }
            }
            let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
            main_body.push_str(&format!("    {}({})\n", sig.name, call_args.join(", ")));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                main_body.push_str(&format!("    println({})\n", p.name));
            }
        } else {
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    val {} = {}\n",
                        p.name,
                        super::render_value(val, unwrap_mut_ref(&p.ty), Language::Kt)
                    ));
                    args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!(
                "    val result = {}({})\n",
                sig.name,
                args.join(", ")
            ));
            main_body.push_str("    println(result)\n");
        }
    }

    let app_kt = format!(
        r#"package codle

fun {}({}){} {{
    TODO()
}}

fun main() {{
{}}}"#,
        sig.name,
        params_str.join(", "),
        ret_str,
        main_body,
    );

    let tests_code = generate_kotlin_tests(sig, &challenge.tests);

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::Kt,
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

gradle init --type kotlin-application --dsl kotlin --project-name "{}" --package codle --no-incubating --overwrite

cat >> app/build.gradle.kts << 'TESTLOG'

tasks.withType<Test> {{
    testLogging {{
        events("passed", "failed", "skipped")
    }}
}}
TESTLOG

cat > app/src/main/kotlin/codle/App.kt << 'SOLUTION'
{}
SOLUTION

mkdir -p app/src/test/kotlin/codle
cat > app/src/test/kotlin/codle/AppTest.kt << 'TESTS'
{}
TESTS

cat > .codle.json << 'METADATA'
{}
METADATA

echo "Run: ./gradlew run"
echo "Test: ./gradlew test"
"#,
        require_commands(&["gradle"]),
        sig.name,
        escape_for_heredoc(&app_kt),
        escape_for_heredoc(&tests_code),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_kotlin_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_fns = Vec::new();

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;
        let mut body = String::new();

        if let Some(inputs) = test.input.as_object() {
            if is_void_with_mut_ref(sig) {
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        val {} = {}\n",
                            p.name,
                            super::render_value(val, inner_ty, Language::Kt)
                        ));
                    }
                }
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
                body.push_str(&format!("        {}({})\n", sig.name, call_args.join(", ")));
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = super::render_value(&test.expected, inner, Language::Kt);
                    body.push_str(&format!("        assertEquals({}, {})\n", expected, p.name));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        val {} = {}\n",
                            p.name,
                            super::render_value(val, unwrap_mut_ref(&p.ty), Language::Kt)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        val result = {}({})\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = super::render_value(&test.expected, &sig.return_type, Language::Kt);
                body.push_str(&format!("        assertEquals({}, result)\n", expected));
            }
        }

        test_fns.push(format!(
            r#"    @Test
    fun test{}() {{
{}    }}"#,
            test_num, body
        ));
    }

    format!(
        r#"package codle

import kotlin.test.Test
import kotlin.test.assertEquals

class AppTest {{
{}
}}"#,
        test_fns.join("\n\n")
    )
}
