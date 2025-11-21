use crate::generator::ir::model::{NumberType, NumberValue};
use crate::generator::ir::types::ResourceType;
use crate::generator::ir::{
    ResourceKey, ResourceKind, ResourceNode, ResourceOrigin,
    ResourceValue,
};
use crate::generator::parsing::{ParsedResource, ScalarValue};
use crate::generator::utils::sanitize_identifier;
use std::str::FromStr;

pub struct NumberTypeHandler;

impl ResourceType for NumberTypeHandler {
    fn name(&self) -> &'static str {
        "number"
    }

    fn xml_tags(&self) -> &'static [&'static str] {
        &["number", "int", "float"]
    }

    fn resource_kind(&self) -> ResourceKind {
        ResourceKind::Number
    }

    fn build_node(
        &self,
        parsed: &ParsedResource,
        origin: ResourceOrigin,
    ) -> Option<ResourceNode> {
        if let ScalarValue::Number {
            value,
            explicit_type,
        } = &parsed.value
        {
            let number_value =
                parse_number_value(value, explicit_type.as_deref())
                    .ok()?;

            Some(ResourceNode {
                kind: ResourceKind::Number,
                value: ResourceValue::Number(number_value),
                origin,
            })
        } else {
            None
        }
    }

    fn emit_rust(
        &self,
        key: &ResourceKey,
        node: &ResourceNode,
        indent: usize,
    ) -> Option<String> {
        if let ResourceValue::Number(number_value) = &node.value {
            let pad = " ".repeat(indent);
            let const_name =
                sanitize_identifier(&key.name).to_uppercase();

            Some(match number_value {
                NumberValue::Int(i) => format!(
                    "{pad}pub const {const_name}: i64 = {i};\n"
                ),
                NumberValue::Float(f) => {
                    let formatted = format_float(*f);
                    format!("{pad}pub const {const_name}: f64 = {formatted};\n")
                }
                NumberValue::BigDecimal(raw) => {
                    let literal = escape_literal(raw);
                    format!(
                        "{pad}pub static {const_name}: std::sync::LazyLock<r_resources::BigDecimal> = std::sync::LazyLock::new(|| {{\n\
                        {pad}    r_resources::BigDecimal::from_str(\"{literal}\").expect(\"valid decimal literal\")\n\
                        {pad}}});\n"
                    )
                }
                NumberValue::Typed { literal, ty } => {
                    format!(
                        "{pad}pub const {const_name}: {} = {};\n",
                        ty.as_str(),
                        literal
                    )
                }
            })
        } else {
            None
        }
    }
}

fn parse_number_value(
    text: &str,
    explicit_type: Option<&str>,
) -> Result<NumberValue, String> {
    let literal = text.trim();
    if literal.is_empty() {
        return Err("Number literal cannot be empty".to_string());
    }

    if let Some(type_hint) = explicit_type {
        return parse_explicit_number(literal, type_hint);
    }

    if looks_like_integer(literal) {
        return literal
            .parse::<i64>()
            .map(NumberValue::Int)
            .or_else(|_| parse_big_decimal(literal));
    }

    // For decimal numbers, check if they require BigDecimal precision
    // f64 has about 15-17 significant digits, so if the number has more,
    // we should use BigDecimal to preserve precision
    if requires_big_decimal_precision(literal) {
        return parse_big_decimal(literal);
    }

    // Try f64 for numbers that fit within its precision
    f64::from_str(literal)
        .map(NumberValue::Float)
        .or_else(|_| parse_big_decimal(literal))
}

fn parse_big_decimal(literal: &str) -> Result<NumberValue, String> {
    bigdecimal::BigDecimal::from_str(literal)
        .map(|_| NumberValue::BigDecimal(literal.to_string()))
        .map_err(|_| format!("Invalid number literal '{literal}'"))
}

fn parse_explicit_number(
    literal: &str,
    type_hint: &str,
) -> Result<NumberValue, String> {
    let type_name = type_hint.trim().to_ascii_lowercase();
    let ty = match type_name.as_str() {
        "i8" => NumberType::I8,
        "i16" => NumberType::I16,
        "i32" => NumberType::I32,
        "i64" => NumberType::I64,
        "u8" => NumberType::U8,
        "u16" => NumberType::U16,
        "u32" => NumberType::U32,
        "u64" => NumberType::U64,
        "f32" => NumberType::F32,
        "f64" => NumberType::F64,
        "bigdecimal" => return parse_big_decimal(literal),
        other => {
            return Err(format!("Unsupported number type '{other}'"))
        }
    };

    let trimmed = literal.trim();
    let formatted = match ty {
        NumberType::I8 => trimmed
            .parse::<i8>()
            .map(|v| v.to_string())
            .map_err(|_| format!("'{trimmed}' does not fit in i8"))?,
        NumberType::I16 => {
            trimmed.parse::<i16>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in i16"),
            )?
        }
        NumberType::I32 => {
            trimmed.parse::<i32>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in i32"),
            )?
        }
        NumberType::I64 => {
            trimmed.parse::<i64>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in i64"),
            )?
        }
        NumberType::U8 => trimmed
            .parse::<u8>()
            .map(|v| v.to_string())
            .map_err(|_| format!("'{trimmed}' does not fit in u8"))?,
        NumberType::U16 => {
            trimmed.parse::<u16>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in u16"),
            )?
        }
        NumberType::U32 => {
            trimmed.parse::<u32>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in u32"),
            )?
        }
        NumberType::U64 => {
            trimmed.parse::<u64>().map(|v| v.to_string()).map_err(
                |_| format!("'{trimmed}' does not fit in u64"),
            )?
        }
        NumberType::F32 => {
            trimmed.parse::<f32>().map(format_float32).map_err(
                |_| format!("'{trimmed}' is not a valid f32 literal"),
            )?
        }
        NumberType::F64 => {
            trimmed.parse::<f64>().map(format_float64).map_err(
                |_| format!("'{trimmed}' is not a valid f64 literal"),
            )?
        }
    };

    Ok(NumberValue::Typed {
        literal: formatted,
        ty,
    })
}

fn looks_like_integer(literal: &str) -> bool {
    !(literal.contains('.')
        || literal.contains('e')
        || literal.contains('E'))
}

/// Check if a decimal number requires BigDecimal precision
/// f64 has about 15-17 significant digits, so numbers with more need BigDecimal
fn requires_big_decimal_precision(literal: &str) -> bool {
    // Count significant digits (all digits except leading/trailing zeros)
    let significant_digits = count_significant_digits(literal);
    // f64 can represent about 15-17 significant digits accurately
    // Use BigDecimal if we have more than 15 significant digits
    significant_digits > 15
}

/// Count significant digits in a number literal
/// This counts all digits, which is a conservative approach
fn count_significant_digits(literal: &str) -> usize {
    let mut digits = 0;
    let mut in_exponent = false;
    
    for ch in literal.chars() {
        match ch {
            '0'..='9' => {
                if !in_exponent {
                    digits += 1;
                }
            }
            'e' | 'E' => {
                in_exponent = true;
            }
            '+' | '-' if in_exponent => {
                // Part of exponent, continue
            }
            '.' => {
                // Decimal point doesn't count as a digit
            }
            _ => {}
        }
    }
    
    digits
}

fn format_float(value: f64) -> String {
    let s = value.to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn format_float32(value: f32) -> String {
    let s = value.to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn format_float64(value: f64) -> String {
    let s = value.to_string();
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        format!("{s}.0")
    }
}

fn escape_literal(literal: &str) -> String {
    literal.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::ir::model::{NumberType, ResourceKind as ModelResourceKind};
    use crate::generator::parsing::{ParsedResource, ResourceKind as AstResourceKind, ScalarValue};
    use std::path::PathBuf;

    // Test parse_number_value for integers
    #[test]
    fn test_parse_integer() {
        let result = parse_number_value("42", None).unwrap();
        assert!(matches!(result, NumberValue::Int(42)));
    }

    #[test]
    fn test_parse_large_integer() {
        let result = parse_number_value("9223372036854775807", None).unwrap(); // i64::MAX
        assert!(matches!(result, NumberValue::Int(9223372036854775807)));
    }

    #[test]
    fn test_parse_integer_too_large_for_i64() {
        let result = parse_number_value("9223372036854775808", None).unwrap(); // i64::MAX + 1
        assert!(matches!(result, NumberValue::BigDecimal(_)));
    }

    #[test]
    fn test_parse_negative_integer() {
        let result = parse_number_value("-42", None).unwrap();
        assert!(matches!(result, NumberValue::Int(-42)));
    }

    // Test parse_number_value for floats
    #[test]
    fn test_parse_float() {
        let result = parse_number_value("3.14", None).unwrap();
        assert!(matches!(result, NumberValue::Float(_)));
        if let NumberValue::Float(f) = result {
            assert!((f - 3.14).abs() < 0.0001);
        }
    }

    #[test]
    fn test_parse_float_scientific_notation() {
        let result = parse_number_value("1.5e10", None).unwrap();
        assert!(matches!(result, NumberValue::Float(_)));
    }

    #[test]
    fn test_parse_float_within_precision() {
        // 15 digits or less should be parsed as f64
        let result = parse_number_value("12345678901234.5", None).unwrap(); // 15 digits
        assert!(matches!(result, NumberValue::Float(_)));
    }

    // Test parse_number_value for BigDecimal (automatic detection)
    #[test]
    fn test_parse_big_decimal_automatic() {
        // More than 15 significant digits should use BigDecimal
        let result = parse_number_value(
            "31212120129092108928901289001982890120988902190812098.218128128191289012077198209812908",
            None,
        )
        .unwrap();
        assert!(matches!(result, NumberValue::BigDecimal(_)));
        if let NumberValue::BigDecimal(s) = result {
            assert!(s.contains("31212120129092108928901289001982890120988902190812098"));
            assert!(s.contains("218128128191289012077198209812908"));
        }
    }

    #[test]
    fn test_parse_big_decimal_many_digits() {
        let result = parse_number_value("123456789012345678901234567890.123456789", None).unwrap();
        assert!(matches!(result, NumberValue::BigDecimal(_)));
    }

    #[test]
    fn test_parse_big_decimal_explicit() {
        let result = parse_number_value("123.456", Some("bigdecimal")).unwrap();
        assert!(matches!(result, NumberValue::BigDecimal(_)));
        if let NumberValue::BigDecimal(s) = result {
            assert_eq!(s, "123.456");
        }
    }

    // Test parse_explicit_number for all numeric types
    #[test]
    fn test_parse_explicit_i8() {
        let result = parse_explicit_number("127", "i8").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::I8, .. }));
        if let NumberValue::Typed { literal, ty: _ } = result {
            assert_eq!(literal, "127");
        }
    }

    #[test]
    fn test_parse_explicit_i8_overflow() {
        let result = parse_explicit_number("128", "i8");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_explicit_i16() {
        let result = parse_explicit_number("32767", "i16").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::I16, .. }));
    }

    #[test]
    fn test_parse_explicit_i32() {
        let result = parse_explicit_number("2147483647", "i32").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::I32, .. }));
    }

    #[test]
    fn test_parse_explicit_i64() {
        let result = parse_explicit_number("9223372036854775807", "i64").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::I64, .. }));
    }

    #[test]
    fn test_parse_explicit_u8() {
        let result = parse_explicit_number("255", "u8").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::U8, .. }));
    }

    #[test]
    fn test_parse_explicit_u16() {
        let result = parse_explicit_number("65535", "u16").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::U16, .. }));
    }

    #[test]
    fn test_parse_explicit_u32() {
        let result = parse_explicit_number("4294967295", "u32").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::U32, .. }));
    }

    #[test]
    fn test_parse_explicit_u64() {
        let result = parse_explicit_number("18446744073709551615", "u64").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::U64, .. }));
    }

    #[test]
    fn test_parse_explicit_f32() {
        let result = parse_explicit_number("3.14", "f32").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::F32, .. }));
        if let NumberValue::Typed { literal, ty: _ } = result {
            assert!(literal.contains("3.14"));
        }
    }

    #[test]
    fn test_parse_explicit_f64() {
        let result = parse_explicit_number("3.141592653589793", "f64").unwrap();
        assert!(matches!(result, NumberValue::Typed { ty: NumberType::F64, .. }));
    }

    #[test]
    fn test_parse_explicit_unsupported_type() {
        let result = parse_explicit_number("123", "invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported number type"));
    }

    // Test looks_like_integer
    #[test]
    fn test_looks_like_integer() {
        assert!(looks_like_integer("42"));
        assert!(looks_like_integer("-42"));
        assert!(looks_like_integer("0"));
        assert!(!looks_like_integer("3.14"));
        assert!(!looks_like_integer("1.5e10"));
        assert!(!looks_like_integer("1E10"));
    }

    // Test requires_big_decimal_precision
    #[test]
    fn test_requires_big_decimal_precision() {
        // 15 digits or less should not require BigDecimal
        assert!(!requires_big_decimal_precision("12345678901234.5")); // 15 digits
        assert!(!requires_big_decimal_precision("1234567890123.56")); // 14 digits
        
        // More than 15 digits should require BigDecimal
        assert!(requires_big_decimal_precision("123456789012345.5")); // 16 digits
        assert!(requires_big_decimal_precision("123456789012345678901234567890.123456789"));
        assert!(requires_big_decimal_precision("31212120129092108928901289001982890120988902190812098.218128128191289012077198209812908"));
    }

    // Test count_significant_digits
    #[test]
    fn test_count_significant_digits() {
        assert_eq!(count_significant_digits("123"), 3);
        assert_eq!(count_significant_digits("123.456"), 6);
        assert_eq!(count_significant_digits("123456789012345"), 15);
        assert_eq!(count_significant_digits("1234567890123456"), 16);
        assert_eq!(count_significant_digits("1.5e10"), 2); // Only digits before 'e'
        assert_eq!(count_significant_digits("123.456e10"), 6);
    }

    // Test format_float
    #[test]
    fn test_format_float() {
        assert_eq!(format_float(3.14), "3.14");
        assert_eq!(format_float(42.0), "42.0");
        // For scientific notation, the format depends on the value and Rust's formatting
        let sci_result = format_float(1.5e10);
        // The result should be a valid number string (could be "15000000000", "1.5e10", "1.5E10", etc.)
        // Just verify it's not empty and can be parsed back
        assert!(!sci_result.is_empty());
        // Verify it can be parsed as a float (this validates the format is correct)
        assert!(sci_result.parse::<f64>().is_ok());
    }

    // Test format_float32
    #[test]
    fn test_format_float32() {
        assert_eq!(format_float32(3.14f32), "3.14");
        assert_eq!(format_float32(42.0f32), "42.0");
    }

    // Test format_float64
    #[test]
    fn test_format_float64() {
        assert_eq!(format_float64(3.14), "3.14");
        assert_eq!(format_float64(42.0), "42.0");
    }

    // Test escape_literal
    #[test]
    fn test_escape_literal() {
        assert_eq!(escape_literal("hello"), "hello");
        assert_eq!(escape_literal(r#"hello"world"#), r#"hello\"world"#);
        assert_eq!(escape_literal(r#"path\to\file"#), r#"path\\to\\file"#);
        assert_eq!(escape_literal(r#"test\"quote"#), r#"test\\\"quote"#);
    }

    // Test parse_number_value with empty string
    #[test]
    fn test_parse_empty_string() {
        let result = parse_number_value("", None);
        assert!(result.is_err());
    }

    // Test parse_number_value with whitespace
    #[test]
    fn test_parse_with_whitespace() {
        let result = parse_number_value("  42  ", None).unwrap();
        assert!(matches!(result, NumberValue::Int(42)));
    }

    // Test emit_rust for Int
    #[test]
    fn test_emit_rust_int() {
        let handler = NumberTypeHandler;
        let key = ResourceKey {
            namespace: vec![],
            name: "test_value".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Number,
            value: ResourceValue::Number(NumberValue::Int(42)),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const TEST_VALUE: i64 = 42;"));
    }

    // Test emit_rust for Float
    #[test]
    fn test_emit_rust_float() {
        let handler = NumberTypeHandler;
        let key = ResourceKey {
            namespace: vec![],
            name: "pi".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Number,
            value: ResourceValue::Number(NumberValue::Float(3.14)),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const PI: f64"));
        assert!(result.contains("3.14"));
    }

    // Test emit_rust for BigDecimal
    #[test]
    fn test_emit_rust_big_decimal() {
        let handler = NumberTypeHandler;
        let key = ResourceKey {
            namespace: vec![],
            name: "big_number".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Number,
            value: ResourceValue::Number(NumberValue::BigDecimal(
                "12345678901234567890.123456789".to_string(),
            )),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub static BIG_NUMBER"));
        assert!(result.contains("LazyLock"));
        assert!(result.contains("BigDecimal"));
        assert!(result.contains("12345678901234567890.123456789"));
        assert!(result.contains("from_str"));
    }

    // Test emit_rust for Typed
    #[test]
    fn test_emit_rust_typed() {
        let handler = NumberTypeHandler;
        let key = ResourceKey {
            namespace: vec![],
            name: "small_int".to_string(),
        };
        let node = ResourceNode {
            kind: ModelResourceKind::Number,
            value: ResourceValue::Number(NumberValue::Typed {
                literal: "127".to_string(),
                ty: NumberType::I8,
            }),
            origin: ResourceOrigin::new(PathBuf::from("test.xml"), false),
        };

        let result = handler.emit_rust(&key, &node, 4).unwrap();
        assert!(result.contains("pub const SMALL_INT: i8 = 127;"));
    }

    // Test build_node
    #[test]
    fn test_build_node() {
        let handler = NumberTypeHandler;
        let parsed = ParsedResource {
            name: "test".to_string(),
            kind: AstResourceKind::Number,
            value: ScalarValue::Number {
                value: "42".to_string(),
                explicit_type: None,
            },
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin).unwrap();
        assert_eq!(result.kind, ModelResourceKind::Number);
        assert!(matches!(
            result.value,
            ResourceValue::Number(NumberValue::Int(42))
        ));
    }

    #[test]
    fn test_build_node_with_explicit_type() {
        let handler = NumberTypeHandler;
        let parsed = ParsedResource {
            name: "test".to_string(),
            kind: AstResourceKind::Number,
            value: ScalarValue::Number {
                value: "127".to_string(),
                explicit_type: Some("i8".to_string()),
            },
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin).unwrap();
        assert_eq!(result.kind, ModelResourceKind::Number);
        assert!(matches!(
            result.value,
            ResourceValue::Number(NumberValue::Typed { .. })
        ));
    }

    #[test]
    fn test_build_node_invalid_number() {
        let handler = NumberTypeHandler;
        let parsed = ParsedResource {
            name: "test".to_string(),
            kind: AstResourceKind::Number,
            value: ScalarValue::Number {
                value: "not_a_number".to_string(),
                explicit_type: None,
            },
        };
        let origin = ResourceOrigin::new(PathBuf::from("test.xml"), false);

        let result = handler.build_node(&parsed, origin);
        assert!(result.is_none());
    }

    // Test name and xml_tags
    #[test]
    fn test_handler_name() {
        let handler = NumberTypeHandler;
        assert_eq!(handler.name(), "number");
    }

    #[test]
    fn test_handler_xml_tags() {
        let handler = NumberTypeHandler;
        let tags = handler.xml_tags();
        assert!(tags.contains(&"number"));
        assert!(tags.contains(&"int"));
        assert!(tags.contains(&"float"));
    }

    #[test]
    fn test_handler_resource_kind() {
        let handler = NumberTypeHandler;
        assert_eq!(handler.resource_kind(), ModelResourceKind::Number);
    }
}
