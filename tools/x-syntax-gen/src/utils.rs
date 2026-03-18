//! Utility functions for syntax generation

/// Escape regular expression special characters
#[allow(dead_code)]
pub fn escape_regex(s: &str) -> String {
    s.replace(r"\", r"\\")
     .replace(r".", r"\.")
     .replace(r"+", r"\+")
     .replace(r"*", r"\*")
     .replace(r"?", r"\?")
     .replace(r"(", r"\(")
     .replace(r")", r"\)")
     .replace(r"[", r"\[")
     .replace(r"]", r"\]")
     .replace(r"{", r"\{")
     .replace(r"}", r"\}")
     .replace(r"^", r"\^")
     .replace(r"$", r"\$")
     .replace(r"-", r"\-")
     .replace(r"|", r"\|")
}

/// Convert camelCase to snake_case
#[allow(dead_code)]
pub fn camel_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
