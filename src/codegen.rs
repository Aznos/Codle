use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::challenge::Challenge;
use crate::language::Language;
use crate::signature::{FunctionSignature, RustType};

/// Write a setup.sh script and make it executable
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

/// Escape content for use in a heredoc - handles single quotes in code
fn escape_for_heredoc(content: &str) -> String {
    content.to_string()
}

/// Generate bash snippet that checks required commands are available
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

pub fn generate_scaffold(
    challenge: &Challenge,
    sig: &FunctionSignature,
    lang: Language,
    output_dir: &Path,
) -> Result<(), String> {
    match lang {
        Language::Rs => generate_rust(challenge, sig, output_dir),
        Language::Py => generate_python(challenge, sig, output_dir),
        Language::Kt => generate_kotlin(challenge, sig, output_dir),
        Language::Java => generate_java(challenge, sig, output_dir),
        Language::C => generate_c(challenge, sig, output_dir),
        Language::Cpp => generate_cpp(challenge, sig, output_dir),
    }
}

fn translate_type(ty: &RustType, lang: Language) -> String {
    match lang {
        Language::Rs => translate_type_rs(ty),
        Language::Py => translate_type_py(ty),
        Language::Kt => translate_type_kt(ty),
        Language::Java => translate_type_java(ty),
        Language::C => translate_type_c(ty),
        Language::Cpp => translate_type_cpp(ty),
    }
}

fn translate_type_rs(ty: &RustType) -> String {
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

fn translate_type_py(ty: &RustType) -> String {
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

fn translate_type_kt(ty: &RustType) -> String {
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

fn translate_type_java(ty: &RustType) -> String {
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

fn translate_type_c(ty: &RustType) -> String {
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

fn translate_type_cpp(ty: &RustType) -> String {
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

fn render_value(value: &Value, ty: &RustType, lang: Language) -> String {
    match lang {
        Language::Rs => render_value_rs(value, ty),
        Language::Py => render_value_py(value, ty),
        Language::Kt => render_value_kt(value, ty),
        Language::Java => render_value_java(value, ty),
        Language::C => render_value_c(value, ty),
        Language::Cpp => render_value_cpp(value, ty),
    }
}

// --- Rust value rendering ---

fn render_value_rs(value: &Value, ty: &RustType) -> String {
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

// --- Python value rendering ---

fn render_value_py(value: &Value, ty: &RustType) -> String {
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

// --- Kotlin value rendering ---

fn render_value_kt(value: &Value, ty: &RustType) -> String {
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

// --- Java value rendering ---

fn render_value_java(value: &Value, ty: &RustType) -> String {
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

// --- C value rendering ---

fn render_value_c(value: &Value, ty: &RustType) -> String {
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

// --- C++ value rendering ---

fn render_value_cpp(value: &Value, ty: &RustType) -> String {
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

// --- Helpers ---

fn has_mut_ref_params(sig: &FunctionSignature) -> bool {
    sig.params.iter().any(|p| matches!(&p.ty, RustType::MutRef(_)))
}

fn is_void_with_mut_ref(sig: &FunctionSignature) -> bool {
    sig.return_type == RustType::Void && has_mut_ref_params(sig)
}

fn get_first_test_inputs(challenge: &Challenge) -> Option<&serde_json::Map<std::string::String, Value>> {
    challenge.tests.first().and_then(|t| t.input.as_object())
}

fn unwrap_mut_ref(ty: &RustType) -> &RustType {
    match ty {
        RustType::MutRef(inner) => inner,
        other => other,
    }
}

// --- Rust generator ---

fn generate_rust(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    // Build function signature
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{}: {}", p.name, translate_type(&p.ty, Language::Rs)))
        .collect();
    let ret_str = if sig.return_type == RustType::Void {
        String::new()
    } else {
        format!(" -> {}", translate_type(&sig.return_type, Language::Rs))
    };

    let mut main_body = std::string::String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            // Declare mutable vars, call function, print result
            for p in &sig.params {
                if let RustType::MutRef(inner) = &p.ty {
                    if let Some(val) = inputs.get(&p.name) {
                        main_body.push_str(&format!(
                            "    let mut {} = {};\n",
                            p.name,
                            render_value(val, inner, Language::Rs)
                        ));
                    }
                } else if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    let {} = {};\n",
                        p.name,
                        render_value(val, &p.ty, Language::Rs)
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
            // Print the first mut ref param
            if let Some(p) = sig.params.iter().find(|p| matches!(&p.ty, RustType::MutRef(_))) {
                main_body.push_str(&format!("    println!(\"{{:?}}\", {});\n", p.name));
            }
        } else {
            // Normal: declare vars, call, print result
            let mut args = Vec::new();
            for p in &sig.params {
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    let {} = {};\n",
                        p.name,
                        render_value(val, &p.ty, Language::Rs)
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

    let main_rs = format!(
        r#"fn {}({}){} {{
    todo!()
}}

fn main() {{
{}}}"#,
        sig.name,
        params_str.join(", "),
        ret_str,
        main_body
    );

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

cargo init --name "{}"

cat > src/main.rs << 'SOLUTION'
{}
SOLUTION

echo "Run: cargo run"
"#,
        require_commands(&["cargo"]),
        sig.name,
        escape_for_heredoc(&main_rs)
    );

    write_setup_script(output_dir, &setup_sh)
}

// --- Python generator ---

fn generate_python(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                translate_type(unwrap_mut_ref(&p.ty), Language::Py)
            )
        })
        .collect();
    let ret_hint = if sig.return_type == RustType::Void {
        " -> None".to_string()
    } else {
        format!(" -> {}", translate_type(&sig.return_type, Language::Py))
    };

    let mut main_body = std::string::String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} = {}\n",
                        p.name,
                        render_value(val, inner_ty, Language::Py)
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
                        render_value(val, unwrap_mut_ref(&p.ty), Language::Py)
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

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

python3 -m venv venv
source venv/bin/activate

cat > requirements.txt << 'EOF'
# Add dependencies here
EOF

pip install -r requirements.txt

cat > solution.py << 'SOLUTION'
{}
SOLUTION

echo "Run: source venv/bin/activate && python solution.py"
"#,
        require_commands(&["python3", "pip"]),
        escape_for_heredoc(&solution_py)
    );

    write_setup_script(output_dir, &setup_sh)
}

// --- Kotlin generator ---

fn generate_kotlin(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| {
            format!(
                "{}: {}",
                p.name,
                translate_type(unwrap_mut_ref(&p.ty), Language::Kt)
            )
        })
        .collect();
    let ret_str = if sig.return_type == RustType::Void {
        String::new()
    } else {
        format!(": {}", translate_type(&sig.return_type, Language::Kt))
    };

    let mut main_body = std::string::String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    val {} = {}\n",
                        p.name,
                        render_value(val, inner_ty, Language::Kt)
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
                        render_value(val, unwrap_mut_ref(&p.ty), Language::Kt)
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

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

gradle init --type kotlin-application --dsl kotlin --project-name "{}" --package codle --no-incubating --overwrite

cat > app/src/main/kotlin/codle/App.kt << 'SOLUTION'
{}
SOLUTION

echo "Run: ./gradlew run"
"#,
        require_commands(&["gradle"]),
        sig.name,
        escape_for_heredoc(&app_kt)
    );

    write_setup_script(output_dir, &setup_sh)
}

// --- Java generator ---

fn generate_java(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| {
            format!(
                "{} {}",
                translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                p.name
            )
        })
        .collect();

    let ret_type = translate_type(&sig.return_type, Language::Java);
    let default_return = match &sig.return_type {
        RustType::Void => String::new(),
        RustType::Bool => "        return false;\n".to_string(),
        RustType::I32 | RustType::Usize => "        return 0;\n".to_string(),
        RustType::F64 => "        return 0.0;\n".to_string(),
        RustType::String => "        return \"\";\n".to_string(),
        RustType::Vec(_) => format!("        return new {};\n", render_value_java(&Value::Array(vec![]), &sig.return_type)),
        _ => "        return null;\n".to_string(),
    };

    let mut main_body = std::string::String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "        {} {} = {};\n",
                        translate_type(inner_ty, Language::Java),
                        p.name,
                        render_value(val, inner_ty, Language::Java)
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
                        translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                        p.name,
                        render_value(val, unwrap_mut_ref(&p.ty), Language::Java)
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

    let setup_sh = format!(
        r#"#!/bin/bash
set -e

{}

gradle init --type java-application --dsl groovy --project-name "{}" --package codle --no-incubating --overwrite

cat > app/src/main/java/codle/App.java << 'SOLUTION'
{}
SOLUTION

echo "Run: ./gradlew run"
"#,
        require_commands(&["gradle"]),
        sig.name,
        escape_for_heredoc(&app_java)
    );

    write_setup_script(output_dir, &setup_sh)
}

// --- C generator ---

/// For C, Vec<T> params expand to (T* name, int name_len)
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

fn generate_c(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
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

    let mut main_body = std::string::String::new();
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
            // Build call args
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
            // Print first mut ref array
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

    let makefile = r#"CC = gcc
CFLAGS = -Wall -Wextra -std=c11 -g
TARGET = solution
SRC = solution.c

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $(TARGET) $(SRC)

run: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(TARGET)

.PHONY: all run clean"#;

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

echo "Run: make && ./solution"
"#,
        require_commands(&["gcc", "make"]),
        makefile,
        escape_for_heredoc(&solution_c)
    );

    write_setup_script(output_dir, &setup_sh)
}

// --- C++ generator ---

fn generate_cpp(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{} {}", translate_type(&p.ty, Language::Cpp), p.name))
        .collect();

    let ret_type = translate_type(&sig.return_type, Language::Cpp);

    let default_return = match &sig.return_type {
        RustType::Void => String::new(),
        RustType::Bool => "    return false;\n".to_string(),
        RustType::I32 | RustType::Usize => "    return 0;\n".to_string(),
        RustType::F64 => "    return 0.0;\n".to_string(),
        RustType::String => "    return \"\";\n".to_string(),
        RustType::Vec(_) => "    return {};\n".to_string(),
        _ => "    return {};\n".to_string(),
    };

    let mut main_body = std::string::String::new();
    if let Some(inputs) = get_first_test_inputs(challenge) {
        if is_void_with_mut_ref(sig) {
            for p in &sig.params {
                let inner_ty = unwrap_mut_ref(&p.ty);
                if let Some(val) = inputs.get(&p.name) {
                    main_body.push_str(&format!(
                        "    {} {} = {};\n",
                        translate_type(inner_ty, Language::Cpp),
                        p.name,
                        render_value(val, inner_ty, Language::Cpp)
                    ));
                }
            }
            let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
            main_body.push_str(&format!("    {}({});\n", sig.name, call_args.join(", ")));
            // Print first mut ref param
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
                        translate_type(unwrap_mut_ref(&p.ty), Language::Cpp),
                        p.name,
                        render_value(val, unwrap_mut_ref(&p.ty), Language::Cpp)
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
    // Check if we need vector or string
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

    let makefile = r#"CXX = g++
CXXFLAGS = -Wall -Wextra -std=c++17 -g
TARGET = solution
SRC = solution.cpp

all: $(TARGET)

$(TARGET): $(SRC)
	$(CXX) $(CXXFLAGS) -o $(TARGET) $(SRC)

run: $(TARGET)
	./$(TARGET)

clean:
	rm -f $(TARGET)

.PHONY: all run clean"#;

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

echo "Run: make && ./solution"
"#,
        require_commands(&["g++", "make"]),
        makefile,
        escape_for_heredoc(&solution_cpp)
    );

    write_setup_script(output_dir, &setup_sh)
}
