use num_bigint::BigInt;
use num_traits::{One, Zero};
use crate::scalar;

#[derive(Debug, Clone)]
pub struct F1Field {
    pub one: BigInt,
    pub zero: BigInt,
    order: BigInt,
    half: BigInt,
    pub negone: BigInt,
}

impl F1Field {
    pub fn new(order: BigInt) -> Self {
        let one = BigInt::one();
        let zero = BigInt::zero();
        let half = &order >> 1;
        let negone = &order - &one;

        Self {
            one,
            zero,
            order,
            half,
            negone,
        }
    }

    pub fn e(&self, res: BigInt) -> BigInt {
        let r = res % &self.order;
        if r < self.zero {
            r + &self.order
        } else {
            r
        }
    }

    pub fn mul(&self, a: &BigInt, b: &BigInt) -> BigInt {
        (a * b) % &self.order
    }

    pub fn sub(&self, a: &BigInt, b: &BigInt) -> BigInt {
        if a >= b {
            a - b
        } else {
            &self.order - b + a
        }
    }

    pub fn add(&self, a: &BigInt, b: &BigInt) -> BigInt {
        let res = a + b;
        if res >= self.order {
            res - &self.order
        } else {
            res
        }
    }

    pub fn inv(&self, a: &BigInt) -> BigInt {
        if a.is_zero() {
            panic!("Zero has no inverse");
        }

        let mut t = self.zero.clone();
        let mut r = self.order.clone();
        let mut newt = self.one.clone();
        let mut newr = a % &self.order;

        while !newr.is_zero() {
            let q = &r / &newr;

            let temp_t = &t - &q * &newt;
            t = newt;
            newt = temp_t;

            let temp_r = &r - &q * &newr;
            r = newr;
            newr = temp_r;
        }

        if t < self.zero {
            t + &self.order
        } else {
            t
        }
    }

    pub fn div(&self, a: &BigInt, b: &BigInt) -> BigInt {
        self.mul(a, &self.inv(b))
    }

    pub fn eq(&self, a: &BigInt, b: &BigInt) -> bool {
        a == b
    }

    pub fn square(&self, a: &BigInt) -> BigInt {
        (a * a) % &self.order
    }

    pub fn lt(&self, a: &BigInt, b: &BigInt) -> bool {
        let aa = if a > &self.half {
            a - &self.order
        } else {
            a.clone()
        };
        let bb = if b > &self.half {
            b - &self.order
        } else {
            b.clone()
        };
        aa < bb
    }

    pub fn geq(&self, a: &BigInt, b: &BigInt) -> bool {
        let aa = if a > &self.half {
            a - &self.order
        } else {
            a.clone()
        };
        let bb = if b > &self.half {
            b - &self.order
        } else {
            b.clone()
        };
        aa >= bb
    }

    pub fn neg(&self, a: &BigInt) -> BigInt {
        if a.is_zero() {
            a.clone()
        } else {
            &self.order - a
        }
    }

    pub fn is_zero(&self, a: &BigInt) -> bool {
        a.is_zero()
    }

    pub fn pow(&self, mut base: BigInt, mut exp: BigInt) -> BigInt {
        if scalar::is_zero(&exp) {
            return self.one.clone();
        }
    
        if exp < self.zero {
            base = self.inv(&base);
            exp = -exp;
        }
    
        let bits = scalar::bits(&exp);
    
        if bits.is_empty() {
            return self.one.clone();
        }
    
        let mut res = base.clone();
        for i in (0..bits.len() - 1).rev() {
            res = self.square(&res);
            if bits[i] == 1 {
                res = self.mul(&res, &base);
            }
        }
    
        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;
    use num_traits::Zero;

    fn field() -> F1Field {
        F1Field::new(BigInt::from(13))
    }

    fn e(f: &F1Field, val: i64) -> BigInt {
        f.e(BigInt::from(val))
    }

    #[test]
    fn creates_finite_field_with_specific_order() {
        let f = field();
        assert_eq!(f.one, BigInt::from(1));
        assert_eq!(f.zero, BigInt::from(0));
        assert_eq!(f.order, BigInt::from(13));
        assert_eq!(f.half, BigInt::from(13 >> 1));
        assert_eq!(f.negone, BigInt::from(12));
    }

    #[test]
    fn maps_value_back_into_field() {
        let f = field();
        assert_eq!(f.e(BigInt::from(26)), BigInt::from(0));
        assert_eq!(f.e(BigInt::from(-2)), BigInt::from(11));
        assert_eq!(f.e(BigInt::from(-15)), BigInt::from(11));
    }

    #[test]
    fn adds_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 20);
        let c = e(&f, 13);

        assert_eq!(f.add(&a, &a), BigInt::from(4));
        assert_eq!(f.add(&b, &a), BigInt::from(9));
        assert_eq!(f.add(&c, &c), BigInt::from(0));
    }

    #[test]
    fn subtracts_within_field() {
        let f = field();
        let a = e(&f, 4);
        let b = e(&f, 2);

        assert_eq!(f.sub(&a, &b), BigInt::from(2));
        assert_eq!(f.sub(&b, &a), BigInt::from(11));
    }

    #[test]
    fn multiplies_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 11);

        assert_eq!(f.mul(&a, &a), BigInt::from(4));
        assert_eq!(f.mul(&a, &b), BigInt::from(9));
    }

    #[test]
    fn divides_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 4);

        assert_eq!(f.div(&a, &b), BigInt::from(7));
    }

    #[test]
    fn compares_eq_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 3);

        assert!(f.eq(&a, &a));
        assert!(!f.eq(&a, &b));
    }

    #[test]
    fn squares_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 5);

        assert_eq!(f.square(&a), BigInt::from(4));
        assert_eq!(f.square(&b), BigInt::from(12));
    }

    #[test]
    fn inverts_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 11);

        assert_eq!(f.inv(&a), BigInt::from(7));
        assert_eq!(f.inv(&b), BigInt::from(6));
    }

    #[test]
    #[should_panic(expected = "Zero has no inverse")]
    fn inv_panics_on_zero() {
        let f = field();
        let _ = f.inv(&BigInt::zero());
    }

    #[test]
    fn compares_lt_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 3);

        assert!(f.lt(&a, &b));
        assert!(!f.lt(&b, &a));
    }

    #[test]
    fn compares_geq_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, 3);

        assert!(!f.geq(&a, &b));
        assert!(f.geq(&b, &a));
    }

    #[test]
    fn negates_within_field() {
        let f = field();
        let a = e(&f, 2);
        let b = e(&f, -3);

        assert_eq!(f.neg(&a), BigInt::from(11));
        assert_eq!(f.neg(&b), BigInt::from(3));
    }

    #[test]
    fn checks_is_zero_within_field() {
        let f = field();
        let a = e(&f, 0);
        let b = e(&f, 1);

        assert!(f.is_zero(&a));
        assert!(!f.is_zero(&b));
    }

    #[test]
    fn exponentiates_within_field() {
        let f = field();
        let a = e(&f, 0);
        let b = e(&f, 1);
        let c = e(&f, 2);
        let d = e(&f, 3);

        assert_eq!(f.pow(b.clone(), a.clone()), BigInt::from(1));
        assert_eq!(f.pow(b.clone(), c.clone()), BigInt::from(1));
        assert_eq!(f.pow(c.clone(), d.clone()), BigInt::from(8));
        assert_eq!(f.pow(c.clone(), BigInt::from(-1)), f.inv(&c));
        assert_eq!(f.pow(d.clone(), BigInt::from(-30)), BigInt::from(1));
    }
}
