use std::ops::Shl;

use num_bigint::{BigInt, RandBigInt, RandomBits};

use crate::keccak::keccak256;

use self::curve::G;

mod curve;

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

pub fn get_address(public_key: &curve::Point) -> String {
    let public_key: Vec<u8> = encode_public_key_uncompressed(public_key)
        .into_iter()
        .skip(1)
        .collect();
    // let public_key = hex::encode(public_key);
    println!("usable pk: {:?}", hex::encode(public_key.clone()));
    let hash: Vec<u8> = keccak256(&public_key).into_iter().skip(12).collect();
    hex::encode(hash)
}

pub fn sign(private_key: &BigInt, message: &[u8]) -> (BigInt, BigInt) {
    unimplemented!()
}

#[cfg(test)]
mod test {
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
}
