use num_bigint::BigInt;
use std::str;
use base64::{engine::general_purpose, Engine as _};

/// Converts a BigInt to a hexadecimal string (no 0x prefix, even-length)
pub fn big_int_to_hex(value: &BigInt) -> String {
    let mut hex = value.to_str_radix(16);
    if hex.len() % 2 != 0 {
        hex = format!("0{hex}");
    }
    hex
}

/// Converts a hexadecimal string (with or without 0x prefix) to a BigInt
pub fn hex_to_big_int(value: &str) -> Result<BigInt, String> {
    let formatted = if value.starts_with("0x") || value.starts_with("0X") {
        value.to_string()
    } else {
        format!("0x{value}")
    };
    BigInt::parse_bytes(formatted.trim_start_matches("0x").as_bytes(), 16)
        .ok_or_else(|| "Invalid hexadecimal string".to_string())
}

/// Converts a big-endian byte slice to a BigInt
pub fn be_bytes_to_big_int(bytes: &[u8]) -> BigInt {
    BigInt::from_bytes_be(num_bigint::Sign::Plus, bytes)
}

/// Converts a little-endian byte slice to a BigInt
pub fn le_bytes_to_big_int(bytes: &[u8]) -> BigInt {
    BigInt::from_bytes_le(num_bigint::Sign::Plus, bytes)
}

/// Converts a BigInt to a big-endian byte vector with optional padding
pub fn big_int_to_be_bytes(value: &BigInt, size: Option<usize>) -> Result<Vec<u8>, String> {
    let bytes = value.to_bytes_be().1;
    let min_size = bytes.len();

    match size {
        Some(s) if s < min_size => Err(format!("Size {s} is too small, need at least {min_size} bytes")),
        Some(s) => {
            let mut padded = vec![0u8; s - min_size];
            padded.extend_from_slice(&bytes);
            Ok(padded)
        },
        None => Ok(bytes),
    }
}

/// Converts a BigInt to a little-endian byte vector with optional padding
pub fn big_int_to_le_bytes(value: &BigInt, size: Option<usize>) -> Result<Vec<u8>, String> {
    let mut bytes = value.to_bytes_le().1;
    let min_size = bytes.len();

    match size {
        Some(s) if s < min_size => Err(format!("Size {s} is too small, need at least {min_size} bytes")),
        Some(s) => {
            bytes.resize(s, 0u8);
            Ok(bytes)
        },
        None => Ok(bytes),
    }
}

/// Converts a byte slice to a hexadecimal string (no 0x prefix)
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Converts a hexadecimal string (no 0x prefix) to a byte vector
pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
    let clean = if hex_str.len() % 2 != 0 {
        format!("0{hex_str}")
    } else {
        hex_str.to_string()
    };
    hex::decode(clean).map_err(|e| format!("Invalid hex: {e}"))
}

/// Converts a byte slice to a base64 string
pub fn bytes_to_base64(bytes: &[u8]) -> String {
    general_purpose::STANDARD.encode(bytes)
}

/// Converts a base64 string to a byte vector
pub fn base64_to_bytes(s: &str) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("Invalid base64: {e}"))
}

/// Converts a UTF-8 string to a base64 string
pub fn text_to_base64(text: &str) -> String {
    general_purpose::STANDARD.encode(text.as_bytes())
}

/// Converts a base64 string to a UTF-8 string
pub fn base64_to_text(s: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD
        .decode(s)
        .map_err(|e| format!("Invalid base64: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_big_int_to_hexadecimal() {
        let value = BigInt::parse_bytes(b"0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f", 16).unwrap();
        let hex = big_int_to_hex(&value);
        assert_eq!(hex, "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    }

    #[test]
    fn test_hexadecimal_to_big_int() {
        let hex_str = "0x0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let bigint = hex_to_big_int(hex_str).unwrap();
        let back = big_int_to_hex(&bigint);
        assert_eq!(back, "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f");
    }

    #[test]
    fn test_be_le_conversion() {
        let bytes: Vec<u8> = (0..=31).collect();
        let be = be_bytes_to_big_int(&bytes);
        let le = le_bytes_to_big_int(&bytes);
        let from_be = big_int_to_be_bytes(&be, Some(32)).unwrap();
        let from_le = big_int_to_le_bytes(&le, Some(32)).unwrap();
        assert_eq!(from_be, bytes);
        assert_eq!(from_le, bytes);
    }

    #[test]
    fn test_hex_to_bytes_and_back() {
        let hex = "01020304";
        let bytes = hex_to_bytes(hex).unwrap();
        let back = bytes_to_hex(&bytes);
        assert_eq!(hex, back);
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = bytes_to_base64(data);
        let decoded = base64_to_bytes(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_base64_text_conversion() {
        let text = "Hello, World!";
        let base64 = text_to_base64(text);
        let result = base64_to_text(&base64).unwrap();
        assert_eq!(text, result);
    }

    #[test]
    fn test_invalid_base64_handling() {
        let garbage = "#@. Unsupported characters .@#";
        let result = base64_to_text(garbage);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_ascii_base64_text_conversion() {
        let text = "🔥 БД Ω 好 ت 本";
        let base64 = text_to_base64(text);
        let decoded = base64_to_text(&base64).unwrap();
        assert_eq!(decoded, text);
    }

    #[test]
    fn test_big_int_padding_too_small() {
        let val = BigInt::parse_bytes(b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f", 16).unwrap();
        let err = big_int_to_be_bytes(&val, Some(20)).unwrap_err();
        assert!(err.contains("too small"));
    }
}
