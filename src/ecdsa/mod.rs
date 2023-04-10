use crate::{
    address::Address,
    ecdsa::curve::{Point, O, P},
    keccak::keccak256,
};
use num_bigint::{BigInt, BigUint, RandBigInt};

use self::{
    curve::G,
    number_theory::{mod_inverse, mod_sqrt, modulo},
};

mod curve;
mod number_theory;

/// **NOTE: This is not cryptographically secure, only for illustrative purposes**
pub fn gen_random_private_key() -> BigInt {
    let mut rand = rand::thread_rng();
    rand.gen_bigint_range(&BigInt::from(0), &curve::O)
}

pub fn get_public_key(private_key: &BigInt) -> curve::Point {
    G.mul(private_key)
}

pub fn encode_public_key_uncompressed(public_key: &curve::Point) -> Vec<u8> {
    // we start with 0x04 to indicate uncompressed format
    let mut result = vec![4u8];

    let mut x_part = public_key.x.to_bytes_be().1;
    let mut y_part = public_key.y.to_bytes_be().1;

    while x_part.len() < 32 {
        x_part.push(0u8);
    }

    while y_part.len() < 32 {
        y_part.push(0u8);
    }

    result.extend_from_slice(&x_part);
    result.extend_from_slice(&y_part);
    result
}

pub fn encode_public_key_compressed(public_key: &curve::Point) -> Vec<u8> {
    let mut result = if public_key.clone().y % 2 == BigInt::from(1) {
        vec![3u8]
    } else {
        vec![2u8]
    };
    result.extend_from_slice(&public_key.x.to_bytes_be().1);
    result
}

pub fn get_address(public_key: &curve::Point) -> Address {
    let public_key: Vec<u8> = encode_public_key_uncompressed(public_key)
        .into_iter()
        .skip(1)
        .collect();
    // let public_key = hex::encode(public_key);
    println!("usable pk: {:?}", hex::encode(public_key.clone()));
    let hash: Vec<u8> = keccak256(&public_key).into_iter().skip(12).collect();
    Address::from(BigUint::from_bytes_be(&hash))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub r: BigInt,
    pub s: BigInt,
    pub v: u8,
}

/// Signs a message with a private key
impl Signature {
    pub fn create(private_key: &BigInt, hash: [u8; 32]) -> Self {
        let mut rand = rand::thread_rng();

        let mut k = BigInt::default();
        let mut R = Point::infinity();

        while R.x == BigInt::default() {
            k = rand.gen_bigint_range(&BigInt::from(0), &curve::O);
            R = G.mul(&k);
        }

        let r = modulo(&R.x, &O);

        let hash = BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash);

        let s = (hash + private_key * r.clone()) * mod_inverse(&k, &O);
        let s = modulo(&s, &O);

        let recovery_id = if R.y % 2 == BigInt::from(0) {
            if R.x < O.clone() {
                0
            } else {
                2
            }
        } else {
            if R.x < O.clone() {
                1
            } else {
                3
            }
        };

        Signature {
            r,
            s,
            v: recovery_id + 27,
        }
    }

    // TODO: implement this and create proper verification
    // pub fn create_with_chain_id(private_key: &BigInt, message: &[u8], chain_id: u8) -> Self {
    //     let signature = Signature::create(private_key, message);
    //     let recovery_id = signature.v - 27;
    //     let v = chain_id * 2 + 35 + recovery_id;

    //     Self {
    //         r: signature.r,
    //         s: signature.s,
    //         v,
    //     }
    // }

    pub fn verify(&self, hash: &[u8], public_key: &curve::Point) -> bool {
        // reject invalid values for parameters
        if self.r < BigInt::from(1) || self.r > O.clone() {
            return false;
        }
        if self.s < BigInt::from(1) || self.s > O.clone() {
            return false;
        }
        if self.v < 27 || self.v > 30 {
            return false;
        }

        let hash = BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash);

        let w = mod_inverse(&self.s, &O);
        let u1 = hash * w.clone();
        let u2 = self.r.clone() * w;

        let p1 = G.mul(&u1);
        let p2 = public_key.mul(&u2);

        let mut r_point = p1.add(&p2);

        if r_point.x == BigInt::default() {
            return false;
        }

        r_point.x = modulo(&r_point.x, &O);

        r_point.x == self.r
    }

    pub fn recover_public_key(&self, hash: &[u8]) -> Point {
        let hash = BigInt::from_bytes_be(num_bigint::Sign::Plus, &hash);

        let recovery_id = self.v - 27;

        let mut x = self.r.clone();
        let mut y = BigInt::default();

        let is_even = recovery_id % 2 == 0;
        let is_over_o = recovery_id > 1;

        if is_over_o {
            x += O.clone();
        }

        let y2 = x.clone() * x.clone() * x.clone() + BigInt::from(7);
        let y_option_1 = mod_sqrt(&y2, &P).unwrap();
        let y_option_2 = P.clone() - y_option_1.clone();

        if is_even && y_option_1.clone() % 2 == BigInt::from(0) {
            y = y_option_1;
        } else if is_even && y_option_2.clone() % 2 == BigInt::from(0) {
            y = y_option_2;
        } else if !is_even && y_option_1.clone() % 2 == BigInt::from(1) {
            y = y_option_1;
        } else if !is_even && y_option_2.clone() % 2 == BigInt::from(1) {
            y = y_option_2;
        } else {
            println!("Could not find y");
            y = y_option_2; // i guess
        }

        let r_point = Point::new(&x, &y);

        let u1 = modulo(&(-hash * mod_inverse(&r_point.x, &O)), &O);
        let u2 = modulo(&(&self.s * mod_inverse(&r_point.x, &O)), &O);

        let p1 = G.mul(&u1);
        let p2 = r_point.mul(&u2);

        p1.add(&p2)
    }

    pub fn ecrecover(&self, hash: &[u8]) -> Address {
        let public_key = self.recover_public_key(hash);
        get_address(&public_key)
    }

    pub fn to_signature_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        let mut r_bytes = self.r.to_bytes_be().1;
        let mut s_bytes = self.s.to_bytes_be().1;

        // ensure that the signature is 65 bytes long
        if r_bytes.len() < 32 {
            let mut new_bytes = vec![0; 32 - r_bytes.len()];
            new_bytes.append(&mut r_bytes);
            r_bytes = new_bytes;
        }

        if s_bytes.len() < 32 {
            let mut new_bytes = vec![0; 32 - s_bytes.len()];
            new_bytes.append(&mut s_bytes);
            s_bytes = new_bytes;
        }

        bytes.append(&mut r_bytes);
        bytes.append(&mut s_bytes);
        bytes.push(self.v);

        bytes
    }

    pub fn from_signature_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 65 {
            return None;
        }

        let r = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[0..32]);
        let s = BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes[32..64]);
        let v = bytes[64];

        Some(Self { r, s, v })
    }
}

#[cfg(test)]
mod test {
    use crate::util::bigint_from_hex;

    use super::*;

    #[test]
    fn get_pubkey_uncompressed() {
        let private_key = BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &hex::decode("b4d39783863980d393ef99e0b68711a407b4cdb92cab6a27899af9a178a01c93")
                .unwrap(),
        );
        let public_key = get_public_key(&private_key);

        assert_eq!(
            hex::encode(encode_public_key_uncompressed(&public_key)),
            "0476abf7ad93d73818541bb7c5e28fa011e2935f5bf507591693da8594efd23a2925e325adae63c1111224e964d5b86d32027b61429ea155adf9edb84e6bb3fd46"
        )
    }

    #[test]
    fn get_pubkey_compressed_even() {
        let private_key = BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &hex::decode("b4d39783863980d393ef99e0b68711a407b4cdb92cab6a27899af9a178a01c93")
                .unwrap(),
        );
        let public_key = get_public_key(&private_key);

        assert_eq!(
            hex::encode(encode_public_key_compressed(&public_key)),
            "0276abf7ad93d73818541bb7c5e28fa011e2935f5bf507591693da8594efd23a29"
        )
    }

    #[test]
    fn get_pubkey_compressed_odd() {
        let private_key = BigInt::from_bytes_be(
            num_bigint::Sign::Plus,
            &hex::decode("7b2f17cf50ef33bcb8b404d718b2e1fde3f2d025fe34f8d3f4c6e526e447ef13")
                .unwrap(),
        );
        let public_key = get_public_key(&private_key);

        assert_eq!(
            hex::encode(encode_public_key_compressed(&public_key)),
            "0314397848a6600eee675f59fb7829917a30c6dca7f1e7c82bdffbb7774978fe98"
        )
    }

    #[test]
    fn sign_ecrecover_self() {
        let private_key =
            bigint_from_hex("c7fb672c8a1ae5a87fbd97bba7aa5a9024dc9dc7a3cfa97b3759af744008195a")
                .unwrap();
        let public_key_point = get_public_key(&private_key);
        let address = get_address(&public_key_point);

        let message = "hello world";
        let hash = keccak256(message.as_bytes());

        let signature = Signature::create(&private_key, hash);
        let recovered_address = signature.ecrecover(&hash);

        assert_eq!(address, recovered_address);
    }

    #[test]
    fn sign_ecrecover_ext() {
        let signature = Signature {
            v: 27,
            r: bigint_from_hex("1556a70d76cc452ae54e83bb167a9041f0d062d000fa0dcb42593f77c544f647")
                .unwrap(),
            s: bigint_from_hex("1643d14dbd6a6edc658f4b16699a585181a08dba4f6d16a9273e0e2cbed622da")
                .unwrap(),
        };
        let hash = hex::decode("3ea2f1d0abf3fc66cf29eebb70cbd4e7fe762ef8a09bcc06c8edf641230afec0")
            .unwrap();

        let recovered_address = signature.ecrecover(&hash);

        assert_eq!(
            recovered_address.to_string(),
            "0x80c67eec6f8518b5bb707ecc718b53782ac71543"
        )
    }

    #[test]
    fn signature_from_bytes() {
        let signature1 = Signature {
            v: 27,
            r: bigint_from_hex("1556a70d76cc452ae54e83bb167a9041f0d062d000fa0dcb42593f77c544f647")
                .unwrap(),
            s: bigint_from_hex("1643d14dbd6a6edc658f4b16699a585181a08dba4f6d16a9273e0e2cbed622da")
                .unwrap(),
        };

        let signature2 = Signature::from_signature_bytes(&signature1.to_signature_bytes())
            .expect("failed to parse signature");

        let signature3 = Signature::from_signature_bytes(&hex::decode("1556a70d76cc452ae54e83bb167a9041f0d062d000fa0dcb42593f77c544f6471643d14dbd6a6edc658f4b16699a585181a08dba4f6d16a9273e0e2cbed622da1b").unwrap()).unwrap();

        assert_eq!(signature1, signature2);
        assert_eq!(signature1, signature3);
    }
}
