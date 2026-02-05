#[derive(Debug, Clone, PartialEq)]
pub enum RustType {
    I32,
    F64,
    Usize,
    Bool,
    String,
    Char,
    Vec(Box<RustType>),
    MutRef(Box<RustType>),
    Void,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: std::string::String,
    pub ty: RustType,
}

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: std::string::String,
    pub params: Vec<Param>,
    pub return_type: RustType,
}

pub fn parse_signature(sig: &str) -> Result<FunctionSignature, std::string::String> {
    let sig = sig.trim();

    // Strip leading "fn "
    let rest = sig
        .strip_prefix("fn ")
        .ok_or_else(|| "Signature must start with 'fn '".to_string())?;

    // Split name from params
    let paren_open = rest
        .find('(')
        .ok_or_else(|| "Missing opening parenthesis".to_string())?;
    let name = rest[..paren_open].trim().to_string();

    // Find matching closing paren
    let paren_close = find_matching_paren(rest, paren_open)?;

    let params_str = &rest[paren_open + 1..paren_close];
    let params = parse_params(params_str)?;

    // Parse return type
    let after_parens = rest[paren_close + 1..].trim();
    let return_type = if after_parens.starts_with("->") {
        let ty_str = after_parens[2..].trim();
        parse_type(ty_str)?
    } else {
        RustType::Void
    };

    Ok(FunctionSignature {
        name,
        params,
        return_type,
    })
}

fn find_matching_paren(s: &str, open: usize) -> Result<usize, std::string::String> {
    let mut depth = 0;
    for (i, c) in s[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(open + i);
                }
            }
            _ => {}
        }
    }
    Err("Unmatched parenthesis".to_string())
}

fn parse_params(params_str: &str) -> Result<Vec<Param>, std::string::String> {
    let trimmed = params_str.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let parts = split_respecting_angle_brackets(trimmed);
    let mut params = Vec::new();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        let colon_pos = part
            .find(':')
            .ok_or_else(|| format!("Missing ':' in parameter: '{}'", part))?;

        let name = part[..colon_pos].trim().to_string();
        let ty_str = part[colon_pos + 1..].trim();
        let ty = parse_type(ty_str)?;

        params.push(Param { name, ty });
    }

    Ok(params)
}

fn split_respecting_angle_brackets(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    parts.push(&s[start..]);
    parts
}

fn parse_type(ty_str: &str) -> Result<RustType, std::string::String> {
    let ty_str = ty_str.trim();

    // Handle &mut T
    if let Some(inner) = ty_str.strip_prefix("&mut ") {
        let inner_type = parse_type(inner.trim())?;
        return Ok(RustType::MutRef(Box::new(inner_type)));
    }

    // Handle Vec<T>
    if let Some(rest) = ty_str.strip_prefix("Vec<") {
        let inner = rest
            .strip_suffix('>')
            .ok_or_else(|| format!("Unclosed Vec<> in type: '{}'", ty_str))?;
        let inner_type = parse_type(inner.trim())?;
        return Ok(RustType::Vec(Box::new(inner_type)));
    }

    // Primitive types
    match ty_str {
        "i32" => Ok(RustType::I32),
        "f64" => Ok(RustType::F64),
        "usize" => Ok(RustType::Usize),
        "bool" => Ok(RustType::Bool),
        "String" => Ok(RustType::String),
        "char" => Ok(RustType::Char),
        _ => Err(format!("Unknown type: '{}'", ty_str)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_sum() {
        let sig = parse_signature("fn two_sum(nums: Vec<i32>, target: i32) -> Vec<usize>").unwrap();
        assert_eq!(sig.name, "two_sum");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.params[0].name, "nums");
        assert_eq!(sig.params[0].ty, RustType::Vec(Box::new(RustType::I32)));
        assert_eq!(sig.params[1].name, "target");
        assert_eq!(sig.params[1].ty, RustType::I32);
        assert_eq!(sig.return_type, RustType::Vec(Box::new(RustType::Usize)));
    }

    #[test]
    fn test_reverse_string() {
        let sig = parse_signature("fn reverse_string(s: &mut Vec<char>)").unwrap();
        assert_eq!(sig.name, "reverse_string");
        assert_eq!(sig.params.len(), 1);
        assert_eq!(
            sig.params[0].ty,
            RustType::MutRef(Box::new(RustType::Vec(Box::new(RustType::Char))))
        );
        assert_eq!(sig.return_type, RustType::Void);
    }

    #[test]
    fn test_is_valid() {
        let sig = parse_signature("fn is_valid(s: String) -> bool").unwrap();
        assert_eq!(sig.name, "is_valid");
        assert_eq!(sig.params[0].ty, RustType::String);
        assert_eq!(sig.return_type, RustType::Bool);
    }

    #[test]
    fn test_find_median() {
        let sig = parse_signature(
            "fn find_median_sorted_arrays(nums1: Vec<i32>, nums2: Vec<i32>) -> f64",
        )
        .unwrap();
        assert_eq!(sig.return_type, RustType::F64);
    }

    #[test]
    fn test_merge() {
        let sig = parse_signature(
            "fn merge(nums1: &mut Vec<i32>, m: i32, nums2: &mut Vec<i32>, n: i32)",
        )
        .unwrap();
        assert_eq!(sig.params.len(), 4);
        assert_eq!(
            sig.params[0].ty,
            RustType::MutRef(Box::new(RustType::Vec(Box::new(RustType::I32))))
        );
        assert_eq!(sig.params[2].name, "nums2");
    }
}
