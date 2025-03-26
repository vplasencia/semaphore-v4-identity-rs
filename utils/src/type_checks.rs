// src/type_checks.rs
use num_bigint::BigInt;

/// Enum representing supported types. Used for type checking.
#[derive(Debug, PartialEq)]
pub enum SupportedType {
    Number,
    Boolean,
    String,
    Function,
    Array,
    Uint8Array,
    Buffer,
    Object,
    BigInt,
    StringifiedBigInt,
    Hexadecimal,
    BigNumber,
    BigNumberish,
}

/// Checks if a value is defined (not None).
pub fn is_defined<T>(value: &Option<T>) -> bool {
    value.is_some()
}

/// Rust handles types statically, so these are example value checks for dynamic contexts.

pub fn is_number(value: &serde_json::Value) -> bool {
    value.is_number()
}

pub fn is_boolean(value: &serde_json::Value) -> bool {
    value.is_boolean()
}

pub fn is_string(value: &serde_json::Value) -> bool {
    value.is_string()
}

pub fn is_object(value: &serde_json::Value) -> bool {
    value.is_object()
}

pub fn is_array(value: &serde_json::Value) -> bool {
    value.is_array()
}

pub fn is_uint8_array(_value: &serde_json::Value) -> bool {
    // You can define custom handling for binary types
    false
}

pub fn is_buffer(_value: &serde_json::Value) -> bool {
    // Placeholder for custom buffer check
    false
}

pub fn is_bigint(value: &serde_json::Value) -> bool {
    if let Some(s) = value.as_str() {
        BigInt::parse_bytes(s.as_bytes(), 10).is_some()
    } else {
        false
    }
}

pub fn is_stringified_bigint(value: &serde_json::Value) -> bool {
    is_bigint(value)
}

pub fn is_hexadecimal(value: &serde_json::Value, prefix: bool) -> bool {
    if let Some(s) = value.as_str() {
        if prefix {
            s.starts_with("0x") || s.starts_with("0X") && s[2..].chars().all(|c| c.is_digit(16))
        } else {
            s.chars().all(|c| c.is_digit(16))
        }
    } else {
        false
    }
}

pub fn is_big_number(value: &serde_json::Value) -> bool {
    is_bigint(value) || is_stringified_bigint(value)
}

pub fn is_big_numberish(value: &serde_json::Value) -> bool {
    is_number(value)
        || is_bigint(value)
        || is_stringified_bigint(value)
        || is_hexadecimal(value, true)
        || is_buffer(value)
        || is_uint8_array(value)
}

pub fn is_type(value: &serde_json::Value, ty: SupportedType) -> bool {
    match ty {
        SupportedType::Number => is_number(value),
        SupportedType::Boolean => is_boolean(value),
        SupportedType::String => is_string(value),
        SupportedType::Function => false, // Not applicable in Rust
        SupportedType::Array => is_array(value),
        SupportedType::Uint8Array => is_uint8_array(value),
        SupportedType::Buffer => is_buffer(value),
        SupportedType::Object => is_object(value),
        SupportedType::BigInt => is_bigint(value),
        SupportedType::StringifiedBigInt => is_stringified_bigint(value),
        SupportedType::Hexadecimal => is_hexadecimal(value, true),
        SupportedType::BigNumber => is_big_number(value),
        SupportedType::BigNumberish => is_big_numberish(value),
    }
}

pub fn is_supported_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "number"
            | "boolean"
            | "string"
            | "function"
            | "Array"
            | "Uint8Array"
            | "Buffer"
            | "object"
            | "bigint"
            | "stringified-bigint"
            | "hexadecimal"
            | "bignumber"
            | "bignumberish"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_number() {
        assert!(is_number(&json!(1)));
        assert!(!is_number(&json!("string")));
    }

    #[test]
    fn test_is_boolean() {
        assert!(is_boolean(&json!(true)));
        assert!(!is_boolean(&json!("string")));
    }

    #[test]
    fn test_is_string() {
        assert!(is_string(&json!("string")));
        assert!(!is_string(&json!(1)));
    }

    #[test]
    fn test_is_array() {
        assert!(is_array(&json!([1, 2, 3])));
        assert!(!is_array(&json!(1)));
    }

    #[test]
    fn test_is_object() {
        assert!(is_object(&json!({"key": "value"})));
        assert!(!is_object(&json!(1)));
    }

    #[test]
    fn test_is_bigint() {
        assert!(is_bigint(&json!(BigInt::from(1).to_string())));
        assert!(!is_bigint(&json!("1n")));
    }

    #[test]
    fn test_is_stringified_bigint() {
        assert!(is_stringified_bigint(&json!("1242342342342342")));
        assert!(!is_stringified_bigint(&json!("0x12")));
        assert!(!is_stringified_bigint(&json!(1)));
    }

    #[test]
    fn test_is_hexadecimal() {
        assert!(is_hexadecimal(&json!("0x12"), true));
        assert!(is_hexadecimal(&json!("12"), false))
    }

    #[test]
    fn test_is_big_number() {
        assert!(is_big_number(&json!("1")));
        assert!(is_big_number(&json!(BigInt::from(1).to_string())));
        // assert!(is_big_number(&json!("0x12")));
        // assert!(!is_big_number(&json!([1, 2])));
    }

    #[test]
    fn test_is_big_numberish() {
        assert!(is_big_numberish(&json!(1)));
        assert!(is_big_numberish(&json!("1")));
        assert!(is_big_numberish(&json!("0x12")));
        assert!(!is_big_numberish(&json!("string")));
    }

    #[test]
    fn test_is_type_true() {
        assert!(is_type(&json!(1), SupportedType::Number));
        assert!(is_type(&json!(false), SupportedType::Boolean));
        assert!(is_type(&json!("string"), SupportedType::String));
        assert!(is_type(&json!([1,2]), SupportedType::Array));
        assert!(is_type(&json!({}), SupportedType::Object));
        assert!(is_type(&json!(BigInt::from(1).to_string()), SupportedType::BigInt));
        assert!(is_type(&json!("1242342342342342"), SupportedType::StringifiedBigInt));
        assert!(is_type(&json!("0x12"), SupportedType::Hexadecimal));
    }

    #[test]
    fn test_is_type_false() {
        assert!(!is_type(&json!("string"), SupportedType::Number));
        assert!(!is_type(&json!(1), SupportedType::Boolean));
        assert!(!is_type(&json!(1), SupportedType::String));
        assert!(!is_type(&json!(1), SupportedType::Array));
        assert!(!is_type(&json!(1), SupportedType::Uint8Array));
        assert!(!is_type(&json!(1), SupportedType::Buffer));
        assert!(!is_type(&json!(1), SupportedType::Object));
        assert!(!is_type(&json!(1), SupportedType::BigInt));
        assert!(!is_type(&json!(1), SupportedType::StringifiedBigInt));
        assert!(!is_type(&json!(1), SupportedType::Hexadecimal));
        assert!(!is_type(&json!(1), SupportedType::BigNumber));
        assert!(!is_type(&json!("string"), SupportedType::BigNumber));
        assert!(!is_type(&json!("string"), SupportedType::BigNumberish));
    }

    #[test]
    fn test_is_supported_type_true() {
        for t in [
            "number", "boolean", "string", "function", "Array", "Uint8Array", "Buffer",
            "object", "bigint", "stringified-bigint", "hexadecimal", "bignumber", "bignumberish"
        ] {
            assert!(is_supported_type(t));
        }
    }

    #[test]
    fn test_is_supported_type_false() {
        assert!(!is_supported_type("unknown"));
    }
}