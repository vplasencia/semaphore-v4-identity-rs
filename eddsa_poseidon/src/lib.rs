mod util_functions;
use util_functions::{Signature, prune_buffer, hash_input};
use baby_jubjub::{BASE8, Fr, Point, add_point, in_curve, mul_point_escalar, pack_point, SUBORDER, unpack_point};
use utils::conversions::{le_bigint_to_bytes, le_bytes_to_bigint};
use ::utils::scalar::{shift_right, mul};
use light_poseidon::{Poseidon, PoseidonHasher};
use std::error::Error;
use num_bigint::{BigInt, BigUint};
use ark_bn254::Fr as Fra;

/// Supported hashing algorithm (only BLAKE1 in this version).
pub enum SupportedHashingAlgorithms {
    Blake1,
}

fn string_to_biguint(num_str: &str) -> BigUint {
    num_str
        .parse()
        .expect("Failed to parse the string into BigUint")
}

pub fn poseidon5(nodes: Vec<String>) -> String {
    let mut poseidon = Poseidon::<Fra>::new_circom(2).unwrap();

    let input1 = ark_bn254::Fr::from(string_to_biguint(&nodes[0]));
    let input2 = ark_bn254::Fr::from(string_to_biguint(&nodes[1]));

    let hash = poseidon.hash(&[input1, input2]).unwrap();

    hash.to_string()
}

fn string_to_bigint(num_str: &str) -> BigInt {
    num_str
        .parse()
        .expect("Failed to parse the string into BigUint")
}

/// Derives a secret scalar from a private key buffer.
pub fn derive_secret_scalar(private_key: &[u8]) -> Result<num_bigint::BigInt, Box<dyn Error>> {
    let mut hash = hash_input(&private_key);
    hash.truncate(32);
    prune_buffer(hash.clone());
    Ok(shift_right(&le_bytes_to_bigint(&hash), &num_bigint::BigInt::from(3)) % &*SUBORDER)
}

/// Derives a public key (a Baby Jubjub point) from a private key buffer.
pub fn derive_public_key(private_key: &[u8]) -> Result<Point, Box<dyn Error>> {
    let s = derive_secret_scalar(private_key)?;
    Ok(mul_point_escalar(&BASE8, s))
}

/// Signs a message using the given private key and Poseidon hash.
pub fn sign_message(private_key: &[u8], message: &[u8]) -> Result<Signature, Box<dyn Error>> {
    let hash = hash_input(&private_key);
    let s_bytes = &mut hash[..32].to_vec();
    prune_buffer(s_bytes.to_vec());
    let s = le_bytes_to_bigint(s_bytes);
    let a = mul_point_escalar(&BASE8, shift_right(&s, &BigInt::from(3)));

    let msg_bigint = le_bytes_to_bigint(message);
    let msg_buff = le_bigint_to_bytes(&msg_bigint, Some(32))?;
    let r_buff = hash_input(&[&hash[32..64], &msg_buff].concat());

    let r = Fr.e(le_bytes_to_bigint(&r_buff));
    let r8 = mul_point_escalar(&BASE8, r.clone());
    let message_bigint = le_bytes_to_bigint(message);
    let hm = poseidon5(vec![
        r8.0.to_string(),
        r8.1.to_string(),
        a.0.to_string(),
        a.1.to_string(),
        message_bigint.to_string(),
    ]);
    let hm_bigint = string_to_bigint(&hm);
    let s_final = Fr.add(&r, &Fr.mul(&hm_bigint, &s));

    Ok(Signature { r8, s: s_final })
}

/// Verifies a signature against a given message and public key.
pub fn verify_signature(message: &[u8], signature: &Signature, public_key: &Point) -> Result<bool, Box<dyn Error>> {
    if !in_curve(&signature.r8) || !in_curve(public_key) {
        return Ok(false);
    }

    let message_bigint = le_bytes_to_bigint(message);
    let hm = poseidon5(vec![
        signature.r8.0.to_string(),
        signature.r8.1.to_string(),
        public_key.0.to_string(),
        public_key.1.to_string(),
        message_bigint.to_string(),
    ]);

    let p_left = mul_point_escalar(&BASE8, signature.s.clone());
    let hm_bigint = string_to_bigint(&hm);
    let p_right = add_point(&signature.r8, &mul_point_escalar(public_key, mul(&hm_bigint, &num_bigint::BigInt::from(8))));

    Ok(Fr.eq(&p_left.0, &p_right.0) && Fr.eq(&p_left.1, &p_right.1))
}

/// Packs a public key into a compressed format (bigint).
pub fn pack_public_key(public_key: &Point) -> Result<num_bigint::BigInt, Box<dyn Error>> {
    if !in_curve(public_key) {
        return Err("Invalid public key".into());
    }
    Ok(pack_point(public_key))
}

/// Unpacks a compressed public key.
pub fn unpack_public_key(packed: &num_bigint::BigInt) -> Result<Point, Box<dyn Error>> {
    unpack_point(packed).ok_or_else(|| "Invalid public key".into())
}

/// Packs a signature into 64-byte format.
pub fn pack_signature(sig: &Signature) -> Result<Vec<u8>, Box<dyn Error>> {
    if !in_curve(&sig.r8) || &sig.s >= &SUBORDER {
        return Err("Invalid signature".into());
    }
    let mut packed = vec![0u8; 64];
    packed[..32].copy_from_slice(&le_bigint_to_bytes(&pack_point(&sig.r8), Some(32))?);
    packed[32..].copy_from_slice(&le_bigint_to_bytes(&sig.s, Some(32))?);
    Ok(packed)
}

/// Unpacks a signature from 64-byte format.
pub fn unpack_signature(packed: &[u8]) -> Result<Signature, Box<dyn Error>> {
    if packed.len() != 64 {
        return Err("Packed signature must be 64 bytes".into());
    }
    let r8 = unpack_point(&le_bytes_to_bigint(&packed[..32]))
        .ok_or_else(|| format!("Invalid packed R8 in signature: {}", hex::encode(&packed[..32])))?;
    let s = le_bytes_to_bigint(&packed[32..]);
    Ok(Signature { r8, s })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_derive_public_key_from_string() {
        let private_key = b"secret";
        let result = derive_public_key(private_key);
        assert!(result.is_ok());
        let public_key = result.unwrap();
        // println!("Point0: {}, Point1: {}", public_key.0, public_key.1);
        assert!(in_curve(&public_key));
    }

    // #[test]
    // fn test_sign_and_verify_message_bigint() {
    //     let private_key = b"secret";
    //     let message = BigInt::from(2);
    //     let public_key = derive_public_key(private_key).unwrap();
    //     let signature = sign_message(private_key, &le_bigint_to_bytes(&message, Some(32)).unwrap()).unwrap();
    //     let verified = verify_signature(&le_bigint_to_bytes(&message, Some(32)).unwrap(), &signature, &public_key).unwrap();
    //     assert!(verified);
    // }

    #[test]
    fn test_pack_and_unpack_public_key() {
        let private_key = b"secret";
        let public_key = derive_public_key(private_key).unwrap();
        let packed = pack_public_key(&public_key).unwrap();
        let unpacked = unpack_public_key(&packed).unwrap();
        assert_eq!(public_key.0, unpacked.0);
        assert_eq!(public_key.1, unpacked.1);
    }

    // #[test]
    // fn test_pack_and_unpack_signature() {
    //     let private_key = b"secret";
    //     let message = BigInt::from(2);
    //     let signature = sign_message(private_key, &le_bigint_to_bytes(&message, Some(32)).unwrap()).unwrap();
    //     let packed = pack_signature(&signature).unwrap();
    //     assert_eq!(packed.len(), 64);
    //     let unpacked = unpack_signature(&packed).unwrap();
    //     assert_eq!(signature.r8.0, unpacked.r8.0);
    //     assert_eq!(signature.r8.1, unpacked.r8.1);
    //     assert_eq!(signature.s, unpacked.s);
    // }

    // #[test]
    // fn test_invalid_signature_unpack() {
    //     let mut invalid = vec![0u8; 64];
    //     invalid[0] = 1; // invalid R8
    //     let result = unpack_signature(&invalid);
    //     assert!(result.is_err());
    // }

    #[test]
    fn test_invalid_signature_length_unpack() {
        let short = vec![0u8; 63];
        let result = unpack_signature(&short);
        assert!(result.is_err());
    }

    #[test]
    fn test_public_key_not_on_curve() {
        let invalid_point = (BigInt::from(0), BigInt::from(3));
        let result = pack_public_key(&invalid_point);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_invalid_curve_point() {
        let private_key = b"secret";
        let mut sig = sign_message(private_key, &le_bigint_to_bytes(&BigInt::from(2), Some(32)).unwrap()).unwrap();
        sig.r8.1 = BigInt::from(3); // not on curve
        let res = pack_signature(&sig);
        assert!(res.is_err());
    }

    #[test]
    fn test_random_private_key_derivation() {
        for _ in 0..10 {
            let mut rng = rand::rng();
            let mut key = [0u8; 32];
            rng.fill(&mut key);
            let public_key = derive_public_key(&key);
            assert!(public_key.is_ok());
        }
    }
}