use serde_json::Value;

use crate::models::{
    Challenge, Difficulty, FunctionSignature, Language, ProjectMetadata, RustType,
    TestCase, metadata_json,
};
use super::{
    write_setup_script, require_commands, escape_for_heredoc,
    is_void_with_mut_ref, get_first_test_inputs, unwrap_mut_ref,
};

pub(super) fn translate_type_java(ty: &RustType) -> String {
    match ty {
        RustType::I32 | RustType::Usize => "int".to_string(),
        RustType::F64 => "double".to_string(),
        RustType::Bool => "boolean".to_string(),
        RustType::String => "String".to_string(),
        RustType::Char => "char".to_string(),
        RustType::Vec(inner) => format!("{}[]", translate_type_java(inner)),
        RustType::MutRef(inner) => translate_type_java(inner),
        RustType::Void => "void".to_string(),
    }
}

pub(super) fn render_value_java(value: &Value, ty: &RustType) -> String {
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
                let items: Vec<String> = arr.iter().map(|v| render_value_java(v, inner)).collect();
                format!("new {}[] {{{}}}", translate_type_java(inner), items.join(", "))
            } else {
                format!("new {}[] {{}}", translate_type_java(inner))
            }
        }
        RustType::MutRef(inner) => render_value_java(value, inner),
        RustType::Void => "".to_string(),
    }
}

pub(super) fn generate_java(
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
                "{} {}",
                super::translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                p.name
            )
        })
        .collect();

    let ret_type = super::translate_type(&sig.return_type, Language::Java);
    let default_return = match &sig.return_type {
        RustType::Void => String::new(),
        RustType::Bool => "        return false;\n".to_string(),
        RustType::I32 | RustType::Usize => "        return 0;\n".to_string(),
        RustType::F64 => "        return 0.0;\n".to_string(),
        RustType::String => "        return \"\";\n".to_string(),
        RustType::Vec(_) => format!("        return new {};\n", render_value_java(&Value::Array(vec![]), &sig.return_type)),
        _ => "        return null;\n".to_string(),
    };

    let mut main_body = String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "        {} {} = {};\n",
                        super::translate_type(inner_ty, Language::Java),
                        p.name,
                        super::render_value(val, inner_ty, Language::Java)
                    ));
                }
            }
            let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
            main_body.push_str(&format!(
                "        {}({});\n",
                sig.name,
                call_args.join(", ")
            ));
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                main_body.push_str(&format!(
                    "        System.out.println(java.util.Arrays.toString({}));\n",
                    p.name
                ));
            }
        } else {
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "        {} {} = {};\n",
                        super::translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                        p.name,
                        super::render_value(val, unwrap_mut_ref(&p.ty), Language::Java)
                    ));
                    args.push(p.name.clone());
                }
            }
            main_body.push_str(&format!(
                "        {} result = {}({});\n",
                ret_type,
                sig.name,
                args.join(", ")
            ));
            if matches!(&sig.return_type, RustType::Vec(_)) {
                main_body.push_str(
                    "        System.out.println(java.util.Arrays.toString(result));\n",
                );
            } else {
                main_body.push_str("        System.out.println(result);\n");
            }
        }
    }

    let app_java = format!(
        r#"package codle;

public class App {{
    public static {} {}({}) {{
{}    }}

    public static void main(String[] args) {{
{}    }}
}}"#,
        ret_type,
        sig.name,
        params_str.join(", "),
        default_return,
        main_body,
    );

    let tests_code = generate_java_tests(sig, &challenge.tests);

    let metadata = ProjectMetadata::new(
        challenge.name.clone(),
        Language::Java,
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

gradle init --type java-application --dsl groovy --project-name "{}" --package codle --no-incubating --overwrite

cat >> app/build.gradle << 'TESTLOG'

test {{
    testLogging {{
        events "passed", "failed", "skipped"
    }}
}}
TESTLOG

cat > app/src/main/java/codle/App.java << 'SOLUTION'
{}
SOLUTION

mkdir -p app/src/test/java/codle
cat > app/src/test/java/codle/AppTest.java << 'TESTS'
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
        escape_for_heredoc(&app_java),
        escape_for_heredoc(&tests_code),
        metadata_content
    );

    write_setup_script(output_dir, &setup_sh)
}

pub(super) fn generate_java_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
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
                            "        {} {} = {};\n",
                            super::translate_type(inner_ty, Language::Java),
                            p.name,
                            super::render_value(val, inner_ty, Language::Java)
                        ));
                    }
                }
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
                body.push_str(&format!(
                    "        App.{}({});\n",
                    sig.name,
                    call_args.join(", ")
                ));
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = super::render_value(&test.expected, inner, Language::Java);
                    body.push_str(&format!(
                        "        assertArrayEquals({}, {});\n",
                        expected, p.name
                    ));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        {} {} = {};\n",
                            super::translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                            p.name,
                            super::render_value(val, unwrap_mut_ref(&p.ty), Language::Java)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        {} result = App.{}({});\n",
                    super::translate_type(&sig.return_type, Language::Java),
                    sig.name,
                    args.join(", ")
                ));
                let expected = super::render_value(&test.expected, &sig.return_type, Language::Java);
                if matches!(&sig.return_type, RustType::Vec(_)) {
                    body.push_str(&format!("        assertArrayEquals({}, result);\n", expected));
                } else {
                    body.push_str(&format!("        assertEquals({}, result);\n", expected));
                }
            }
        }

        test_fns.push(format!(
            r#"    @Test
    void test{}() {{
{}    }}"#,
            test_num, body
        ));
    }

    format!(
        r#"package codle;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

class AppTest {{
{}
}}"#,
        test_fns.join("\n\n")
    )
}
