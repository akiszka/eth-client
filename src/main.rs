#![allow(dead_code)]

use ecdsa::gen_random_private_key;

use crate::ecdsa::{get_public_key, get_address};

mod address;
mod ecdsa;
mod keccak;
mod trie;
mod rlp;
mod util;

fn main() {
    let private_key = gen_random_private_key();
    println!("Private key: {}", private_key);

    let public_key = get_public_key(&private_key);
    println!("Address: {}", get_address(&public_key));

    let message = "Hello, world!".as_bytes();
    let hash = keccak::keccak256(message);
    let signature = ecdsa::Signature::create(&private_key, hash);

    println!("Signature: {}", hex::encode(signature.to_signature_bytes()));

    println!("Ecrecover: {}", signature.ecrecover(&hash));
}
