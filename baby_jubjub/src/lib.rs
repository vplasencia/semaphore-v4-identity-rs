mod sqrt; 

use num_bigint::BigInt;
use utils::conversions::{le_bigint_to_bytes, le_bytes_to_bigint};
use utils::scalar;
use crate::sqrt::tonelli_shanks;
use utils::f1_field::F1Field;

pub type Point = (BigInt, BigInt);

lazy_static::lazy_static! {
    pub static ref R: BigInt = BigInt::parse_bytes(b"21888242871839275222246405745257275088548364400416034343698204186575808495617", 10).unwrap();
    pub static ref ORDER: BigInt = BigInt::parse_bytes(b"21888242871839275222246405745257275088614511777268538073601725287587578984328", 10).unwrap();
    pub static ref SUBORDER: BigInt = scalar::shift_right(&ORDER, &BigInt::from(3));
    pub static ref Fr: F1Field = F1Field::new(R.clone());
    pub static ref BASE8: Point = (
        Fr.e(BigInt::parse_bytes(b"5299619240641551281634865583518297030282874472190772894086521144482721001553", 10).unwrap()),
        Fr.e(BigInt::parse_bytes(b"16950150798460657717958625567821834550301663161624707787222815936182638968203", 10).unwrap())
    );
    pub static ref A: BigInt = Fr.e(BigInt::from(168700));
    pub static ref D: BigInt = Fr.e(BigInt::from(168696));
}

pub fn add_point(p1: &Point, p2: &Point) -> Point {
    let beta = Fr.mul(&p1.0, &p2.1);
    let gamma = Fr.mul(&p1.1, &p2.0);
    let delta = Fr.mul(&Fr.sub(&p1.1, &Fr.mul(&A, &p1.0)), &Fr.add(&p2.0, &p2.1));

    let tau = Fr.mul(&beta, &gamma);
    let dtau = Fr.mul(&D, &tau);

    let x3 = Fr.div(&Fr.add(&beta, &gamma), &Fr.add(&Fr.one, &dtau));
    let y3 = Fr.div(&Fr.add(&delta, &Fr.sub(&Fr.mul(&A, &beta), &gamma)), &Fr.sub(&Fr.one, &dtau));

    (x3, y3)
}

pub fn mul_point_escalar(base: &Point, mut e: BigInt) -> Point {
    let mut res = (Fr.zero.clone(), Fr.one.clone());
    let mut exp = base.clone();

    while !scalar::is_zero(&e) {
        if scalar::is_odd(&e) {
            res = add_point(&res, &exp);
        }
        exp = add_point(&exp, &exp);
        e = scalar::shift_right(&e, &BigInt::from(1));
    }

    res
}

pub fn in_curve(p: &Point) -> bool {
    let x2 = Fr.square(&p.0);
    let y2 = Fr.square(&p.1);
    Fr.eq(&Fr.add(&Fr.mul(&A, &x2), &y2), &Fr.add(&Fr.one, &Fr.mul(&Fr.mul(&x2, &y2), &D)))
}

pub fn pack_point(p: &Point) -> BigInt {
    let mut buffer = le_bigint_to_bytes(&p.1, Some(32)).unwrap();
    if Fr.lt(&p.0, &Fr.zero) {
        buffer[31] |= 0x80;
    }
    le_bytes_to_bigint(&buffer)
}

pub fn unpack_point(packed: &BigInt) -> Option<Point> {
    let mut buffer = le_bigint_to_bytes(packed, Some(32)).ok()?;
    let mut sign = false;
    if buffer[31] & 0x80 != 0 {
        sign = true;
        buffer[31] &= 0x7f;
    }

    let y = le_bytes_to_bigint(&buffer);
    if scalar::gt(&y, &R) {
        return None;
    }

    let y2 = Fr.square(&y);
    let den = Fr.sub(&A, &Fr.mul(&D, &y2));
    let num = Fr.sub(&Fr.one, &y2);

    let mut x = tonelli_shanks(&Fr.div(&num, &den), &R)?;
    if sign {
        x = Fr.neg(&x);
    }

    Some((x, y))
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::{Zero, One};

    #[test]
    fn test_add_point_on_curve() {
        let p1 = (Fr.zero.clone(), Fr.one.clone());
        let result = add_point(&p1, &BASE8);
        assert!(in_curve(&result));
    }

    #[test]
    fn test_scalar_multiplication() {
        let scalar = BigInt::from(324);
        let pubkey = mul_point_escalar(&BASE8, scalar.clone());
        assert!(in_curve(&pubkey));
    }

    #[test]
    fn test_pack_point_structure() {
        let scalar = BigInt::from(324);
        let pubkey = mul_point_escalar(&BASE8, scalar);
        let packed = pack_point(&pubkey);
        let stripped = &packed & !(BigInt::one() << 255u32);
        assert_eq!(stripped, pubkey.1);
    }

    #[test]
    fn test_unpack_point_matches_original() {
        let scalar = BigInt::from(324);
        let pubkey = mul_point_escalar(&BASE8, scalar);
        let packed = pack_point(&pubkey);
        let unpacked = unpack_point(&packed).unwrap();
        assert_eq!(unpacked.0, pubkey.0);
        assert_eq!(unpacked.1, pubkey.1);
    }

    #[test]
    fn test_unpack_custom_point() {
        let pubkey = (
            BigInt::parse_bytes(b"10207164244839265210731148792003399330071235260758262804307337735329782473514", 10).unwrap(),
            BigInt::parse_bytes(b"4504034976288485670718230979254896078098063043333320048161019268102694534400", 10).unwrap(),
        );
        let packed = pack_point(&pubkey);
        let unpacked = unpack_point(&packed).unwrap();
        assert_eq!(unpacked.0, pubkey.0);
        assert_eq!(unpacked.1, pubkey.1);
    }

    #[test]
    fn test_unpack_invalid_y_fails() {
        let pubkey = (
            BigInt::parse_bytes(b"10207164244839265210731148792003399330071235260758262804307337735329782473514", 10).unwrap(),
            &*R + BigInt::one(),
        );
        let packed = pack_point(&pubkey);
        assert!(unpack_point(&packed).is_none());
    }

    #[test]
    fn test_tonelli_shanks_zero() {
        let result = tonelli_shanks(&BigInt::zero(), &BigInt::one());
        assert_eq!(result.unwrap(), BigInt::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn test_tonelli_shanks_divide_by_zero() {
        let _ = tonelli_shanks(&BigInt::one(), &BigInt::zero());
    }

    #[test]
    fn test_tonelli_shanks_no_root() {
        let result = tonelli_shanks(&BigInt::from(-1), &BigInt::one());
        assert!(result.is_none());
    }
}
