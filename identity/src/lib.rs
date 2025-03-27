use ark_bn254::Fr as Fra;
use baby_jubjub::Point;
use eddsa_poseidon::util_functions::Signature;
use eddsa_poseidon::{
    derive_public_key, derive_secret_scalar, sign_message as eddsa_sign_message,
    verify_signature as eddsa_verify_signature,
};
use light_poseidon::{Poseidon, PoseidonHasher};
use num_bigint::{BigInt, BigUint};
use rand::Rng;
use std::error::Error;
use utils::conversions::{base64_to_buffer, buffer_to_base64, text_to_base64};

fn string_to_biguint(num_str: &str) -> BigUint {
    num_str
        .parse()
        .expect("Failed to parse the string into BigUint")
}

pub fn poseidon2(nodes: Vec<String>) -> String {
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

#[derive(Debug, Clone)]
pub struct Identity {
    private_key: Vec<u8>,
    secret_scalar: num_bigint::BigInt,
    public_key: Point,
    commitment: num_bigint::BigInt,
}

impl Identity {
    /// Creates a new Semaphore identity instance from a private key (optional).
    pub fn new(private_key: Option<Vec<u8>>) -> Result<Self, Box<dyn Error>> {
        let mut rng = rand::rng();
        let mut key = [0u8; 32];
        rng.fill(&mut key);
        let private_key = private_key.unwrap_or_else(|| (&key).to_vec());
        let secret_scalar = derive_secret_scalar(&private_key)?;
        let public_key = derive_public_key(&private_key)?;
        let public_key_strings = vec![public_key.0.to_string(), public_key.1.to_string()];
        let commitment = string_to_bigint(&poseidon2(public_key_strings));

        Ok(Self {
            private_key,
            secret_scalar,
            public_key,
            commitment,
        })
    }

    /// Returns the private key.
    pub fn private_key(&self) -> &Vec<u8> {
        &self.private_key
    }

    /// Returns the secret scalar.
    pub fn secret_scalar(&self) -> &num_bigint::BigInt {
        &self.secret_scalar
    }

    /// Returns the public key.
    pub fn public_key(&self) -> &Point {
        &self.public_key
    }

    /// Returns the identity commitment.
    pub fn commitment(&self) -> &num_bigint::BigInt {
        &self.commitment
    }

    /// Exports the private key as base64.
    pub fn export(&self) -> String {
        if let Ok(utf8_str) = std::str::from_utf8(&self.private_key) {
            text_to_base64(utf8_str)
        } else {
            buffer_to_base64(&self.private_key)
        }
    }

    /// Imports an identity from a base64-encoded private key.
    pub fn import(encoded: &str) -> Result<Self, Box<dyn Error>> {
        let private_key = base64_to_buffer(encoded)?;
        Identity::new(Some(private_key))
    }

    /// Signs a message with the private key.
    pub fn sign_message(
        &self,
        message: &[u8],
    ) -> Result<Signature, Box<dyn Error>> {
        eddsa_sign_message(&self.private_key, &message)
    }

    /// Verifies a signature with the given public key.
    pub fn verify_signature(
        message: &[u8],
        signature: &Signature,
        public_key: &Point,
    ) -> Result<bool, Box<dyn Error>> {
        eddsa_verify_signature(&message, signature, public_key)
    }

    /// Generates a commitment from a given public key.
    pub fn generate_commitment(public_key: &Point) -> num_bigint::BigInt {
        let public_key_strings = vec![public_key.0.to_string(), public_key.1.to_string()];
        string_to_bigint(&poseidon2(public_key_strings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_random() {
        let identity = Identity::new(None).unwrap();
        let pk = identity.private_key();
        assert_eq!(pk.len(), 32);
        assert_eq!(identity.secret_scalar(), &derive_secret_scalar(&pk).unwrap());
        assert_eq!(identity.public_key(), &derive_public_key(&pk).unwrap());
        let public_key_strings = vec![
            identity.public_key().0.to_string(),
            identity.public_key().1.to_string(),
        ];
        let computed_commitment = string_to_bigint(&poseidon2(public_key_strings));
        assert_eq!(identity.commitment(), &computed_commitment);
    }

    #[test]
    fn test_identity_from_string() {
        let pk = Some(b"secret".to_vec()).unwrap();
        let identity = Identity::new(Some(pk.clone())).unwrap();
        assert_eq!(identity.secret_scalar(), &derive_secret_scalar(&pk).unwrap());
        assert_eq!(identity.public_key(), &derive_public_key(&pk).unwrap());
        let public_key_strings = vec![
            identity.public_key().0.to_string(),
            identity.public_key().1.to_string(),
        ];
        let computed_commitment = string_to_bigint(&poseidon2(public_key_strings));
        println!("PrivateKey: {:?}, SecretScalar: {:?}, Commitment: {:?}", identity.private_key(), identity.secret_scalar(), identity.commitment());
        assert_eq!(identity.commitment(), &computed_commitment);
    }

    #[test]
    fn test_identity_export_import() {
        let identity = Identity::new(Some(b"some key".to_vec())).unwrap();
        let exported = identity.export();
        let imported = Identity::import(&exported).unwrap();
        assert_eq!(imported.private_key(), identity.private_key());
        assert_eq!(imported.secret_scalar(), identity.secret_scalar());
        assert_eq!(imported.public_key(), identity.public_key());
        assert_eq!(imported.commitment(), identity.commitment());
    }

    // #[test]
    // fn test_sign_and_verify() {
    //     let identity = Identity::new(Some(b"verify key".to_vec())).unwrap();
    //     let msg = BigInt::from(42);
    //     let msg_bytes = msg.to_bytes_be().1; // Convert BigInt to a byte array
    //     let sig = identity.sign_message(&msg_bytes).unwrap();
    //     let verification_result = Identity::verify_signature(&msg_bytes, &sig, &identity.public_key());
    //     assert!(verification_result.unwrap_or(false));
    // }

    #[test]
    fn test_commitment_generation() {
        let identity = Identity::new(Some(b"commit test".to_vec())).unwrap();
        let c = Identity::generate_commitment(&identity.public_key());
        assert_eq!(c, identity.commitment().clone());
    }
} 