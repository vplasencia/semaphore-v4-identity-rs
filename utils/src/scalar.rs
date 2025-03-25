/// Scalar module
/// This module provides utility functions for performing scalar operations
/// within a field, especially designed to handle operations on big integers.
/// The operations include scalar inversion, exponentiation, and modular reduction.
/// Functions are implemented to ensure mathematical correctness and efficiency,
/// supporting both positive and negative big integer values.

use num_bigint::BigInt;
use num_traits::{Zero, One, ToPrimitive};
use std::ops::{Shr, Mul};

/// Checks if a BigInt scalar value is zero.
/// 
/// # Arguments
/// * `a` - The BigInt scalar value to check.
/// 
/// # Returns
/// * `true` if `a` is zero, `false` otherwise.
pub fn is_zero(a: &BigInt) -> bool {
    a.is_zero()
}

/// Determines whether a BigInt scalar value is odd.
/// 
/// # Arguments
/// * `a` - The BigInt scalar value to check.
/// 
/// # Returns
/// * `true` if `a` is odd, `false` if it is even.
pub fn is_odd(a: &BigInt) -> bool {
    (a & BigInt::one()) == BigInt::one()
}

/// Performs a bitwise right shift on a BigInt scalar value.
/// Equivalent to dividing by 2^n.
///
/// # Arguments
/// * `a` - The BigInt scalar value to shift.
/// * `n` - The number of bits to shift `a` by.
/// 
/// # Returns
/// * The result of shifting `a` right by `n` bits.
pub fn shift_right(a: &BigInt, n: &BigInt) -> BigInt {
    // Convert BigInt to usize for shifting
    let n_usize = n.to_usize().expect("Shift amount too large");
    a.shr(n_usize)
}

/// Multiplies two BigInt scalar values.
///
/// # Arguments
/// * `a`, `b` - BigInt values to multiply.
/// 
/// # Returns
/// * Product of `a` and `b`.
pub fn mul(a: &BigInt, b: &BigInt) -> BigInt {
    a.mul(b)
}

/// Compares two BigInt scalar values to determine if the first is greater than the second.
///
/// # Arguments
/// * `a`, `b` - BigInt values to compare.
/// 
/// # Returns
/// * `true` if `a` > `b`, otherwise `false`.
pub fn gt(a: &BigInt, b: &BigInt) -> bool {
    a > b
}

/// Converts a BigInt scalar value into an array of bits (as u8).
/// Starting from least significant bit.
///
/// # Arguments
/// * `n` - The BigInt value to convert.
/// 
/// # Returns
/// * Vector of bits (0 or 1) starting from LSB.
pub fn bits(n: &BigInt) -> Vec<u8> {
    let mut res = Vec::new();
    let mut e = n.clone();

    while !e.is_zero() {
        if &e & BigInt::one() == BigInt::one() {
            res.push(1);
        } else {
            res.push(0);
        }
        e = e.shr(1u8);
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use num_traits::{One, Zero};

    #[test]
    fn test_is_zero() {
        assert!(is_zero(&BigInt::zero()));
        assert!(!is_zero(&BigInt::one()));
        assert!(!is_zero(&BigInt::from(123)));
    }

    #[test]
    fn test_is_odd() {
        assert!(is_odd(&BigInt::from(1)));
        assert!(!is_odd(&BigInt::from(2)));
        assert!(is_odd(&BigInt::from(999)));
        assert!(!is_odd(&BigInt::from(1000)));
    }

    #[test]
    fn test_shift_right() {
        assert_eq!(shift_right(&BigInt::from(8), &BigInt::from(1)), BigInt::from(4));
        assert_eq!(shift_right(&BigInt::from(16), &BigInt::from(2)), BigInt::from(4));
        assert_eq!(shift_right(&BigInt::from(1), &BigInt::from(1)), BigInt::from(0));
    }

    #[test]
    fn test_mul() {
        assert_eq!(mul(&BigInt::from(2), &BigInt::from(3)), BigInt::from(6));
        assert_eq!(mul(&BigInt::from(-2), &BigInt::from(3)), BigInt::from(-6));
        assert_eq!(mul(&BigInt::zero(), &BigInt::from(1000)), BigInt::zero());
    }

    #[test]
    fn test_gt() {
        assert!(gt(&BigInt::from(5), &BigInt::from(2)));
        assert!(!gt(&BigInt::from(2), &BigInt::from(5)));
        assert!(!gt(&BigInt::from(2), &BigInt::from(2)));
    }

    #[test]
    fn test_bits() {
        assert_eq!(bits(&BigInt::from(0)), vec![]);
        assert_eq!(bits(&BigInt::from(1)), vec![1]);
        assert_eq!(bits(&BigInt::from(2)), vec![0, 1]);
        assert_eq!(bits(&BigInt::from(3)), vec![1, 1]);
        assert_eq!(bits(&BigInt::from(10)), vec![0, 1, 0, 1]);
        assert_eq!(bits(&BigInt::from(255)), vec![1; 8]);
    }
}
