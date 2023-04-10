/// implementation of secp256k1
/// we only implement this curve since this is used in Ethereum
use std::ops::Mul;

use num_bigint::{BigInt, Sign};
use once_cell::sync::Lazy;

// secp256k1 is y^2 = x^3 + 7

/// the curve is mod P
pub static P: Lazy<BigInt> = Lazy::new(|| {
    BigInt::from_bytes_be(
        num_bigint::Sign::Plus,
        hex::decode("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f")
            .unwrap()
            .as_slice(),
    )
});

/// number of points on the curve obtainable by the generator
pub static O: Lazy<BigInt> = Lazy::new(|| {
    BigInt::from_bytes_be(
        num_bigint::Sign::Plus,
        hex::decode("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141")
            .unwrap()
            .as_slice(),
    )
});

/// the curve generator point
pub static G: Lazy<Point> = Lazy::new(|| {
    Point::new(
        BigInt::from_bytes_be(
            Sign::Plus,
            hex::decode("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
                .unwrap()
                .as_slice(),
        ),
        BigInt::from_bytes_be(
            Sign::Plus,
            hex::decode("483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
                .unwrap()
                .as_slice(),
        ),
    )
});

#[derive(Debug, Clone)]
pub struct Point {
    pub x: BigInt,
    pub y: BigInt,
}

impl Point {
    pub fn new(x: BigInt, y: BigInt) -> Self {
        Self {
            x: mod_p(x),
            y: mod_p(y),
        }
    }

    pub fn from_hex(x: &str, y: &str) -> Self {
        Point::new(
            BigInt::from_bytes_be(Sign::Plus, hex::decode(x).unwrap().as_slice()),
            BigInt::from_bytes_be(Sign::Plus, hex::decode(y).unwrap().as_slice()),
        )
    }

    pub fn infinity() -> Self {
        Point::new(BigInt::from(0), BigInt::from(0))
    }

    pub fn is_on_curve(&self) -> bool {
        unimplemented!()
    }

    pub fn inverse(&self) -> Point {
        Point::new(self.x.clone(), self.y.clone().mul(-1))
    }

    pub fn add(&self, q: &Point) -> Point {
        if *self == Point::infinity() {
            return q.clone();
        }

        if *q == Point::infinity() {
            return self.clone();
        }

        if *self == self.inverse() {
            return Point::infinity();
        }

        let lambda;

        if *self == *q {
            // to avoid division by zero we have a special case for point doubling
            let numerator = mod_p(3 * self.x.pow(2));
            let denominator = mod_p(2 * self.y.clone());
            lambda = mod_p(numerator * mod_inverse(&denominator, &P));
        } else {
            // lambda = mpdmod_p() * mod_inverse(&(), &P);
            let numerator = mod_p(q.clone().y - self.clone().y);
            let denominator = mod_p(q.clone().x - self.clone().x);
            lambda = mod_p(numerator * mod_inverse(&denominator, &P));
        }

        let xr = lambda.clone().pow(2) - q.clone().x - self.clone().x;
        let yr = lambda * (self.clone().x - xr.clone()) - self.clone().y;

        Point::new(xr, yr)
    }

    pub fn mul(&self, a: &BigInt) -> Point {
        if a.sign() == Sign::Minus {
            let inv = self.inverse();
            inv.mul(&(a * -1))
        } else if *a == BigInt::from(0) {
            Point::infinity()
        } else if *a == BigInt::from(1) {
            self.clone()
        } else if a.clone() % 2 == BigInt::from(1) {
            self.add(&self.mul(&(a - 1)))
        } else {
            let double = self.add(self);
            double.mul(&(a / 2))
        }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        return self.x.eq(&other.x) && self.y.eq(&other.y);
    }
}

impl Eq for Point {}

/// calculate a value mod p
fn mod_p(val: BigInt) -> BigInt {
    let mut result = val % P.clone();

    while result.lt(&BigInt::from(0)) {
        result += P.clone();
    }

    result
}

fn mod_inverse(n: &BigInt, p: &BigInt) -> BigInt {
    if p == &BigInt::from(1) {
        return BigInt::from(1);
    }

    let (mut a, mut m, mut x, mut inv) = (n.clone(), p.clone(), BigInt::from(0), BigInt::from(1));

    while a > BigInt::from(1) {
        let div = a.clone() / m.clone();
        let rem = a.clone() % m.clone();
        inv -= div * &x;
        a = rem;
        std::mem::swap(&mut a, &mut m);
        std::mem::swap(&mut x, &mut inv);
    }

    if inv < BigInt::from(0) {
        inv += p
    }

    inv
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn mod_inverse_base() {
        assert_eq!(
            BigInt::from(12),
            mod_inverse(&BigInt::from(10), &BigInt::from(17))
        )
    }

    #[test]
    fn add_infs() {
        assert_eq!(Point::infinity(), Point::infinity().add(&Point::infinity()))
    }

    #[test]
    fn add_symmetry() {
        assert_eq!(G.clone(), G.clone().add(&Point::infinity()));

        assert_eq!(G.clone(), Point::infinity().add(&G.clone()))
    }

    #[test]
    fn add_double() {
        let double_g = Point::from_hex(
            "C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5",
            "1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A",
        );

        assert_eq!(double_g, G.add(&G.clone()));
        assert_eq!(double_g, G.mul(&BigInt::from(2)))
    }

    #[test]
    fn mul_13() {
        let expected = Point::from_hex(
            "F28773C2D975288BC7D1D205C3748651B075FBC6610E58CDDEEDDF8F19405AA8",
            "0AB0902E8D880A89758212EB65CDAF473A1A06DA521FA91F29B5CB52DB03ED81",
        );

        assert_eq!(expected, G.mul(&BigInt::from(13)))
    }

    #[test]
    fn mul_16() {
        let expected = Point::from_hex(
            "E60FCE93B59E9EC53011AABC21C23E97B2A31369B87A5AE9C44EE89E2A6DEC0A",
            "F7E3507399E595929DB99F34F57937101296891E44D23F0BE1F32CCE69616821",
        );

        assert_eq!(expected, G.mul(&BigInt::from(16)))
    }

    #[test]
    fn mul_20() {
        let expected = Point::from_hex(
            "4CE119C96E2FA357200B559B2F7DD5A5F02D5290AFF74B03F3E471B273211C97",
            "12BA26DCB10EC1625DA61FA10A844C676162948271D96967450288EE9233DC3A",
        );

        assert_eq!(expected, G.mul(&BigInt::from(20)))
    }

    #[test]
    fn mul_large1() {
        let expected = Point::from_hex(
            "2F8BDE4D1A07209355B4A7250A5C5128E88B84BDDC619AB7CBA8D569B240EFE4",
            "2753DDD9C91A1C292B24562259363BD90877D8E454F297BF235782C459539959",
        );

        assert_eq!(expected, G.mul(&BigInt::from_str("115792089237316195423570985008687907852837564279074904382605163141518161494332").unwrap()))
    }

    #[test]
    fn mul_large2() {
        let expected = Point::from_hex(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "B7C52588D95C3B9AA25B0403F1EEF75702E84BB7597AABE663B82F6F04EF2777",
        );

        assert_eq!(expected, G.mul(&BigInt::from_str("115792089237316195423570985008687907852837564279074904382605163141518161494336").unwrap()))
    }
}
