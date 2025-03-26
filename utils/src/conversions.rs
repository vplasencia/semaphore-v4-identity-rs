use base64::{engine::general_purpose, Engine as _};
use hex::{decode as hex_decode, encode as hex_encode};
use num_bigint::BigInt;

/// Converts a BigInt to a hexadecimal string (without "0x" prefix)
pub fn bigint_to_hex(value: &BigInt) -> String {
    let mut hex = value.to_str_radix(16);
    if hex.len() % 2 != 0 {
        hex.insert(0, '0');
    }
    hex
}

/// Converts a hexadecimal string to a BigInt (with or without "0x")
pub fn hex_to_big_int(value: &str) -> Result<BigInt, String> {
    let hex = if value.starts_with("0x") || value.starts_with("0X") {
        &value[2..]
    } else {
        value
    };

    if hex.chars().all(|c| c.is_ascii_hexdigit()) {
        BigInt::parse_bytes(hex.as_bytes(), 16).ok_or_else(|| "Invalid hex string".to_string())
    } else {
        Err("Parameter 'value' is not a hexadecimal string".to_string())
    }
}

/// Converts a byte slice to a BigInt (big-endian)
pub fn be_bytes_to_bigint(bytes: &[u8]) -> BigInt {
    BigInt::parse_bytes(hex_encode(bytes).as_bytes(), 16).unwrap()
}

/// Converts a byte slice to a BigInt (little-endian)
pub fn le_bytes_to_bigint(bytes: &[u8]) -> BigInt {
    BigInt::parse_bytes(hex_encode(bytes.iter().rev().cloned().collect::<Vec<u8>>()).as_bytes(), 16).unwrap()
}

/// Converts a BigInt to bytes (big-endian) with optional size padding
pub fn be_bigint_to_bytes(value: &BigInt, size: Option<usize>) -> Result<Vec<u8>, String> {
    let mut hex = bigint_to_hex(value);
    let min_size = (hex.len() + 1) / 2;
    let size = size.unwrap_or(min_size);

    if size < min_size {
        return Err(format!("Size {} is too small, need at least {} bytes", size, min_size));
    }

    while hex.len() < size * 2 {
        hex.insert(0, '0');
    }

    hex_decode(hex).map_err(|e| e.to_string())
}

/// Converts a BigInt to bytes (little-endian) with optional size padding
pub fn le_bigint_to_bytes(value: &BigInt, size: Option<usize>) -> Result<Vec<u8>, String> {
    let mut buf = be_bigint_to_bytes(value, size)?;
    buf.reverse();
    Ok(buf)
}

/// Converts a byte slice to a hexadecimal string
pub fn buffer_to_hex(buffer: &[u8]) -> String {
    hex_encode(buffer)
}

/// Converts a hexadecimal string to a byte buffer
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    let clean = if hex.len() % 2 != 0 {
        format!("0{}", hex)
    } else {
        hex.to_string()
    };

    hex_decode(clean).map_err(|_| "Invalid hexadecimal string".to_string())
}

/// Converts a byte slice to base64 string
pub fn buffer_to_base64(buffer: &[u8]) -> String {
    general_purpose::STANDARD.encode(buffer)
}

/// Converts a base64 string to a byte buffer
pub fn base64_to_buffer(value: &str) -> Result<Vec<u8>, String> {
    general_purpose::STANDARD.decode(value).map_err(|e| e.to_string())
}

/// Converts UTF-8 text to base64
pub fn text_to_base64(value: &str) -> String {
    general_purpose::STANDARD.encode(value.as_bytes())
}

/// Converts base64 to UTF-8 text
pub fn base64_to_text(value: &str) -> Result<String, String> {
    let bytes = base64_to_buffer(value)?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}


#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn test_bigint_to_hexadecimal() {
        let value = BigInt::parse_bytes(
            b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            16,
        )
        .unwrap();
        let hex = bigint_to_hex(&value);
        assert_eq!(
            hex,
            "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"
        );
    }

    #[test]
    fn test_hexadecimal_to_bigint_prefixed_and_unprefixed() {
        let hex_prefixed = "0x1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let hex_unprefixed = "1f1e1d1c1b1a191817161514131211100f0e0d0c0b0a09080706050403020100";
        let expected = BigInt::parse_bytes(hex_unprefixed.as_bytes(), 16).unwrap();

        assert_eq!(hex_to_big_int(hex_prefixed).unwrap(), expected);
        assert_eq!(hex_to_big_int(hex_unprefixed).unwrap(), expected);
    }

    #[test]
    fn test_be_and_le_buffer_to_bigint() {
        let buffer: Vec<u8> = (0..=31).collect();
        let le = le_bytes_to_bigint(&buffer);
        let be = be_bytes_to_bigint(&buffer);

        let back_le = le_bigint_to_bytes(&le, Some(32)).unwrap();
        let back_be = be_bigint_to_bytes(&be, Some(32)).unwrap();

        assert_eq!(buffer, back_le);
        assert_eq!(buffer, back_be);
    }

    #[test]
    fn test_buffer_to_and_from_hexadecimal() {
        let hex = "deadbeef";
        let buffer = hex_to_bytes(hex).unwrap();
        let hex_roundtrip = buffer_to_hex(&buffer);
        assert_eq!(hex_roundtrip, hex);
    }

    #[test]
    fn test_base64_text_roundtrip() {
        let text = "Hello, World!";
        let b64 = text_to_base64(text);
        let roundtrip = base64_to_text(&b64).unwrap();
        assert_eq!(text, roundtrip);
    }

    #[test]
    fn test_base64_binary_roundtrip() {
        let data = b"binary\0data\x1b";
        let b64 = buffer_to_base64(data);
        let decoded = base64_to_buffer(&b64).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_invalid_hex_to_bigint() {
        assert!(hex_to_big_int("thisisnothex").is_err());
    }

    #[test]
    fn test_le_bigint_padding_error() {
        let value = BigInt::from(123456u64);
        let result = le_bigint_to_bytes(&value, Some(1));
        assert!(result.is_err());
    }
}
