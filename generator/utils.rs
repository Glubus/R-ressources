//! Utility functions for code generation.

/// Sanitizes an identifier to be a valid Rust identifier
///
/// Replaces non-alphanumeric characters (except underscores) with underscores
pub fn sanitize_identifier(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("hello-world"), "hello_world");
        assert_eq!(sanitize_identifier("app.name"), "app_name");
        assert_eq!(sanitize_identifier("my_var"), "my_var");
        assert_eq!(sanitize_identifier("test123"), "test123");
    }
}
