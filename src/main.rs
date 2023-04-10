use crate::{ecdsa::{gen_random_private_key, get_public_key, encode_public_key_uncompressed, get_address}, util::bigint_from_hex};

mod address;
mod ecdsa;
mod keccak;
mod trie;
mod util;

fn main() {
    // let private_key = gen_random_private_key();
    let private_key = bigint_from_hex("c7fb672c8a1ae5a87fbd97bba7aa5a9024dc9dc7a3cfa97b3759af744008195a").unwrap();
    let public_key_point = get_public_key(&private_key);
    let public_key = encode_public_key_uncompressed(&public_key_point);
    let address = get_address(&public_key_point);

    println!("Hello, world!");
    println!("{}", hex::encode(private_key.to_bytes_be().1));
    println!("{}", hex::encode(public_key));
    println!("addy: {}", address);
}
