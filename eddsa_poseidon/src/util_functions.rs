use num_bigint::BigInt;
use baby_jubjub::Point;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub r8: Point,
    pub s: BigInt,
}

pub fn prune_buffer(mut buff: Vec<u8>) -> Vec<u8> {
    buff[0] &= 0xf8;
    buff[31] &= 0x7f;
    buff[31] |= 0x40;
    buff
}

pub fn hash_input(message: &[u8]) -> Vec<u8> {
    let mut hash = [0; 64];
    blake::hash(512, message, &mut hash).unwrap();
    hash.to_vec()
}
