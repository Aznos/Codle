use crate::challenge::TestCase;
use crate::codegen::{render_value, translate_type};
use crate::language::Language;
use crate::signature::{FunctionSignature, RustType};

fn unwrap_mut_ref(ty: &RustType) -> &RustType {
    match ty {
        RustType::MutRef(inner) => inner,
        other => other,
    }
}

fn is_void_with_mut_ref(sig: &FunctionSignature) -> bool {
    sig.return_type == RustType::Void
        && sig.params.iter().any(|p| matches!(&p.ty, RustType::MutRef(_)))
}

pub fn generate_rust_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_fns = Vec::new();

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;
        let mut body = String::new();

        if let Some(inputs) = test.input.as_object() {
            if is_void_with_mut_ref(sig) {
                // Declare mutable vars
                for p in &sig.params {
                    if let RustType::MutRef(inner) = &p.ty {
                        if let Some(val) = inputs.get(&p.name) {
                            body.push_str(&format!(
                                "        let mut {} = {};\n",
                                p.name,
                                render_value(val, inner, Language::Rs)
                            ));
                        }
                    } else if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        let {} = {};\n",
                            p.name,
                            render_value(val, &p.ty, Language::Rs)
                        ));
                    }
                }
                // Call function
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
                // Assert on first mut ref param
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = render_value(&test.expected, inner, Language::Rs);
                    body.push_str(&format!("        assert_eq!({}, {});\n", p.name, expected));
                }
            } else {
                // Normal case
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        let {} = {};\n",
                            p.name,
                            render_value(val, &p.ty, Language::Rs)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        let result = {}({});\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = render_value(&test.expected, &sig.return_type, Language::Rs);
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

pub fn generate_python_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_fns = Vec::new();
    test_fns.push(format!("from solution import {}\n", sig.name));

    for (i, test) in tests.iter().enumerate() {
        let test_num = i + 1;
        let mut body = String::new();

        if let Some(inputs) = test.input.as_object() {
            if is_void_with_mut_ref(sig) {
                // Declare vars
                for p in &sig.params {
                    let inner_ty = unwrap_mut_ref(&p.ty);
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "    {} = {}\n",
                            p.name,
                            render_value(val, inner_ty, Language::Py)
                        ));
                    }
                }
                // Call function
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
                body.push_str(&format!("    {}({})\n", sig.name, call_args.join(", ")));
                // Assert on first mut ref param
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = render_value(&test.expected, inner, Language::Py);
                    body.push_str(&format!("    assert {} == {}\n", p.name, expected));
                }
            } else {
                // Normal case
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "    {} = {}\n",
                            p.name,
                            render_value(val, unwrap_mut_ref(&p.ty), Language::Py)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "    result = {}({})\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = render_value(&test.expected, &sig.return_type, Language::Py);
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

pub fn generate_kotlin_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
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
                            render_value(val, inner_ty, Language::Kt)
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
                    let expected = render_value(&test.expected, inner, Language::Kt);
                    body.push_str(&format!("        assertEquals({}, {})\n", expected, p.name));
                }
            } else {
                let mut args = Vec::new();
                for p in &sig.params {
                    if let Some(val) = inputs.get(&p.name) {
                        body.push_str(&format!(
                            "        val {} = {}\n",
                            p.name,
                            render_value(val, unwrap_mut_ref(&p.ty), Language::Kt)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        val result = {}({})\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = render_value(&test.expected, &sig.return_type, Language::Kt);
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

pub fn generate_java_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
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
                            translate_type(inner_ty, Language::Java),
                            p.name,
                            render_value(val, inner_ty, Language::Java)
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
                    let expected = render_value(&test.expected, inner, Language::Java);
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
                            translate_type(unwrap_mut_ref(&p.ty), Language::Java),
                            p.name,
                            render_value(val, unwrap_mut_ref(&p.ty), Language::Java)
                        ));
                        args.push(p.name.clone());
                    }
                }
                body.push_str(&format!(
                    "        {} result = App.{}({});\n",
                    translate_type(&sig.return_type, Language::Java),
                    sig.name,
                    args.join(", ")
                ));
                let expected = render_value(&test.expected, &sig.return_type, Language::Java);
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

pub fn generate_c_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_code = String::new();
    test_code.push_str("#include <stdio.h>\n");
    test_code.push_str("#include <stdbool.h>\n");
    test_code.push_str("#include <stdlib.h>\n");
    test_code.push_str("#include <string.h>\n\n");

    // Forward declare the function
    test_code.push_str(&format!(
        "// Forward declaration - implemented in solution.c\n"
    ));

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
                            let arr_val = render_value(val, inner_ty, Language::C);
                            let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                            test_code.push_str(&format!(
                                "        {} {}_arr[] = {};\n",
                                translate_type(elem, Language::C),
                                p.name,
                                arr_val
                            ));
                            test_code.push_str(&format!("        int {}_len = {};\n", p.name, len));
                            call_args.push(format!("{}_arr", p.name));
                            call_args.push(format!("{}_len", p.name));
                        } else {
                            test_code.push_str(&format!(
                                "        {} {} = {};\n",
                                translate_type(inner_ty, Language::C),
                                p.name,
                                render_value(val, inner_ty, Language::C)
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

                // Check result
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
                                    render_value(expected_val, &RustType::I32, Language::C)
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
                            let arr_val = render_value(val, inner_ty, Language::C);
                            let len = val.as_array().map(|a| a.len()).unwrap_or(0);
                            test_code.push_str(&format!(
                                "        {} {}_arr[] = {};\n",
                                translate_type(elem, Language::C),
                                p.name,
                                arr_val
                            ));
                            test_code.push_str(&format!("        int {}_len = {};\n", p.name, len));
                            call_args.push(format!("{}_arr", p.name));
                            call_args.push(format!("{}_len", p.name));
                        } else {
                            test_code.push_str(&format!(
                                "        {} {} = {};\n",
                                translate_type(inner_ty, Language::C),
                                p.name,
                                render_value(val, inner_ty, Language::C)
                            ));
                            call_args.push(p.name.clone());
                        }
                    }
                }

                // Call function and check result
                match &sig.return_type {
                    RustType::Vec(inner) => {
                        test_code.push_str(&format!(
                            "        {}* result = {}({});\n",
                            translate_type(inner, Language::C),
                            sig.name,
                            call_args.join(", ")
                        ));
                        if let Some(expected_arr) = test.expected.as_array() {
                            test_code.push_str("        int test_passed = 1;\n");
                            for (j, expected_val) in expected_arr.iter().enumerate() {
                                test_code.push_str(&format!(
                                    "        if (result[{}] != {}) test_passed = 0;\n",
                                    j,
                                    render_value(expected_val, inner, Language::C)
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
                            translate_type(&sig.return_type, Language::C),
                            sig.name,
                            call_args.join(", ")
                        ));
                        let expected = render_value(&test.expected, &sig.return_type, Language::C);
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

pub fn generate_cpp_tests(sig: &FunctionSignature, tests: &[TestCase]) -> String {
    let mut test_code = String::new();
    test_code.push_str("#include <iostream>\n");
    test_code.push_str("#include <vector>\n");
    test_code.push_str("#include <string>\n\n");

    // Forward declare the function
    let params_str: Vec<String> = sig
        .params
        .iter()
        .map(|p| format!("{} {}", translate_type(&p.ty, Language::Cpp), p.name))
        .collect();
    let ret_type = translate_type(&sig.return_type, Language::Cpp);
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
                            translate_type(inner_ty, Language::Cpp),
                            p.name,
                            render_value(val, inner_ty, Language::Cpp)
                        ));
                    }
                }
                let call_args: Vec<String> = sig.params.iter().map(|p| p.name.clone()).collect();
                test_code.push_str(&format!(
                    "        {}({});\n",
                    sig.name,
                    call_args.join(", ")
                ));

                // Check result
                if let Some(p) = sig
                    .params
                    .iter()
                    .find(|p| matches!(&p.ty, RustType::MutRef(_)))
                {
                    let inner = unwrap_mut_ref(&p.ty);
                    let expected = render_value(&test.expected, inner, Language::Cpp);
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
                            translate_type(unwrap_mut_ref(&p.ty), Language::Cpp),
                            p.name,
                            render_value(val, unwrap_mut_ref(&p.ty), Language::Cpp)
                        ));
                        args.push(p.name.clone());
                    }
                }
                test_code.push_str(&format!(
                    "        auto result = {}({});\n",
                    sig.name,
                    args.join(", ")
                ));
                let expected = render_value(&test.expected, &sig.return_type, Language::Cpp);
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
