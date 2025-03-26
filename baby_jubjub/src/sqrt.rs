use num_bigint::BigInt;
use crate::F1Field;

/// Tonelli-Shanks square root implementation for a given field order.
/// Only works with the specific prime modulus that the constants are tuned for.
/// See: https://eprint.iacr.org/2012/685.pdf
pub fn tonelli_shanks(n: &BigInt, order: &BigInt) -> Option<BigInt> {
    let fr = F1Field::new(order.clone());

    let sqrt_s = 28;
    let sqrt_z = BigInt::parse_bytes(b"5978345932401256595026418116861078668372907927053715034645334559810731495452", 10).unwrap();
    let sqrt_tm1d2 = BigInt::parse_bytes(b"40770029410420498293352137776570907027550720424234931066070132305055", 10).unwrap();

    if fr.is_zero(n) {
        return Some(fr.zero.clone());
    }

    let mut w = fr.pow(n.clone(), sqrt_tm1d2.clone());
    let a0 = fr.pow(fr.mul(&fr.square(&w), n), BigInt::from(1u64 << (sqrt_s - 1)));

    if fr.eq(&a0, &fr.negone) {
        return None;
    }

    let mut v = sqrt_s;
    let mut x = fr.mul(n, &w);
    let mut b = fr.mul(&x, &w);
    let mut z = sqrt_z;

    while !fr.eq(&b, &fr.one) {
        let mut b2k = fr.square(&b);
        let mut k = 1;
        while !fr.eq(&b2k, &fr.one) {
            b2k = fr.square(&b2k);
            k += 1;
        }

        w = z.clone();
        for _ in 0..(v - k - 1) {
            w = fr.square(&w);
        }

        z = fr.square(&w);
        b = fr.mul(&b, &z);
        x = fr.mul(&x, &w);
        v = k;
    }

    Some(if fr.geq(&x, &fr.zero) { x } else { fr.neg(&x) })
}
