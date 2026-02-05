use std::fs;
use std::path::Path;

use serde_json::Value;

use crate::challenge::Challenge;
use crate::language::Language;
use crate::signature::{FunctionSignature, RustType};

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
    let src_dir = output_dir.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
"#,
        sig.name
    );
    fs::write(output_dir.join("Cargo.toml"), cargo_toml)
        .map_err(|e| format!("Failed to write Cargo.toml: {}", e))?;

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
{}}}
"#,
        sig.name,
        params_str.join(", "),
        ret_str,
        main_body
    );

    fs::write(src_dir.join("main.rs"), main_rs)
        .map_err(|e| format!("Failed to write main.rs: {}", e))?;

    Ok(())
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

    let content = format!(
        r#"def {}({}){}:
    pass


if __name__ == "__main__":
{}
"#,
        sig.name,
        params_str.join(", "),
        ret_hint,
        main_body,
    );

    fs::write(output_dir.join("solution.py"), content)
        .map_err(|e| format!("Failed to write solution.py: {}", e))?;

    // Create requirements.txt (empty, for future dependencies)
    fs::write(output_dir.join("requirements.txt"), "# Add dependencies here\n")
        .map_err(|e| format!("Failed to write requirements.txt: {}", e))?;

    // Create setup script for venv
    let setup_sh = r#"#!/bin/bash
set -e

echo "Creating Python virtual environment..."
python3 -m venv venv

echo "Activating virtual environment..."
source venv/bin/activate

echo "Installing dependencies..."
pip install -r requirements.txt

echo ""
echo "Setup complete! To activate the virtual environment, run:"
echo "  source venv/bin/activate"
echo ""
echo "To run your solution:"
echo "  python solution.py"
"#;
    fs::write(output_dir.join("setup.sh"), setup_sh)
        .map_err(|e| format!("Failed to write setup.sh: {}", e))?;

    // Make setup.sh executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(output_dir.join("setup.sh"))
            .map_err(|e| format!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        fs::set_permissions(output_dir.join("setup.sh"), perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }

    Ok(())
}

// --- Kotlin generator ---

fn generate_kotlin(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    // Create Gradle project structure
    let src_dir = output_dir.join("src").join("main").join("kotlin");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

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

    let content = format!(
        r#"fun {}({}){} {{
    TODO()
}}

fun main() {{
{}}}
"#,
        sig.name,
        params_str.join(", "),
        ret_str,
        main_body,
    );

    fs::write(src_dir.join("Solution.kt"), content)
        .map_err(|e| format!("Failed to write Solution.kt: {}", e))?;

    // Create build.gradle.kts
    let build_gradle = format!(
        r#"plugins {{
    kotlin("jvm") version "1.9.22"
    application
}}

group = "codle"
version = "1.0-SNAPSHOT"

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation(kotlin("test"))
}}

tasks.test {{
    useJUnitPlatform()
}}

kotlin {{
    jvmToolchain(17)
}}

application {{
    mainClass.set("SolutionKt")
}}
"#
    );
    fs::write(output_dir.join("build.gradle.kts"), build_gradle)
        .map_err(|e| format!("Failed to write build.gradle.kts: {}", e))?;

    // Create settings.gradle.kts
    let settings_gradle = format!(
        r#"rootProject.name = "{}"
"#,
        sig.name
    );
    fs::write(output_dir.join("settings.gradle.kts"), settings_gradle)
        .map_err(|e| format!("Failed to write settings.gradle.kts: {}", e))?;

    // Create .gitignore
    let gitignore = r#".gradle/
build/
.idea/
*.iml
"#;
    fs::write(output_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    Ok(())
}

// --- Java generator ---

fn generate_java(
    challenge: &Challenge,
    sig: &FunctionSignature,
    output_dir: &Path,
) -> Result<(), String> {
    // Create Gradle project structure
    let src_dir = output_dir.join("src").join("main").join("java");
    fs::create_dir_all(&src_dir).map_err(|e| format!("Failed to create src dir: {}", e))?;

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

    let content = format!(
        r#"public class Solution {{
    public static {} {}({}) {{
{}    }}

    public static void main(String[] args) {{
{}    }}
}}
"#,
        ret_type,
        sig.name,
        params_str.join(", "),
        default_return,
        main_body,
    );

    fs::write(src_dir.join("Solution.java"), content)
        .map_err(|e| format!("Failed to write Solution.java: {}", e))?;

    // Create build.gradle
    let build_gradle = format!(
        r#"plugins {{
    id 'java'
    id 'application'
}}

group = 'codle'
version = '1.0-SNAPSHOT'

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation 'org.junit.jupiter:junit-jupiter:5.10.0'
}}

test {{
    useJUnitPlatform()
}}

java {{
    toolchain {{
        languageVersion = JavaLanguageVersion.of(17)
    }}
}}

application {{
    mainClass = 'Solution'
}}
"#
    );
    fs::write(output_dir.join("build.gradle"), build_gradle)
        .map_err(|e| format!("Failed to write build.gradle: {}", e))?;

    // Create settings.gradle
    let settings_gradle = format!(
        r#"rootProject.name = '{}'
"#,
        sig.name
    );
    fs::write(output_dir.join("settings.gradle"), settings_gradle)
        .map_err(|e| format!("Failed to write settings.gradle: {}", e))?;

    // Create .gitignore
    let gitignore = r#".gradle/
build/
.idea/
*.iml
"#;
    fs::write(output_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    Ok(())
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

    let content = format!(
        r#"{includes}
{ret_type} {name}({params}) {{
{default_return}}}

int main() {{
{main_body}    return 0;
}}
"#,
        includes = includes,
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
        main_body = main_body,
    );

    fs::write(output_dir.join("solution.c"), content)
        .map_err(|e| format!("Failed to write solution.c: {}", e))?;

    // Create Makefile
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

.PHONY: all run clean
"#;
    fs::write(output_dir.join("Makefile"), makefile)
        .map_err(|e| format!("Failed to write Makefile: {}", e))?;

    // Create .gitignore
    let gitignore = r#"solution
*.o
*.dSYM/
"#;
    fs::write(output_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    Ok(())
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

    let content = format!(
        r#"{includes}

{ret_type} {name}({params}) {{
{default_return}}}

int main() {{
{main_body}    return 0;
}}
"#,
        includes = includes.join("\n"),
        ret_type = ret_type,
        name = sig.name,
        params = params_str.join(", "),
        default_return = default_return,
        main_body = main_body,
    );

    fs::write(output_dir.join("solution.cpp"), content)
        .map_err(|e| format!("Failed to write solution.cpp: {}", e))?;

    // Create Makefile
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

.PHONY: all run clean
"#;
    fs::write(output_dir.join("Makefile"), makefile)
        .map_err(|e| format!("Failed to write Makefile: {}", e))?;

    // Create .gitignore
    let gitignore = r#"solution
*.o
*.dSYM/
"#;
    fs::write(output_dir.join(".gitignore"), gitignore)
        .map_err(|e| format!("Failed to write .gitignore: {}", e))?;

    Ok(())
}
