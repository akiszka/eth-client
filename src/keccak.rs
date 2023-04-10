use tiny_keccak::{Hasher, Keccak};

pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut keccak = Keccak::v256();
    let mut result = [0u8; 32];
    keccak.update(data);
    keccak.finalize(&mut result);
    result
}

pub fn keccak512(data: &[u8]) -> [u8; 64] {
    let mut keccak = Keccak::v512();
    let mut result = [0u8; 64];
    keccak.update(data);
    keccak.finalize(&mut result);
    result
}

#[cfg(test)]
mod test {
    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let expected =
            hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad")
                .unwrap();
        assert_eq!(super::keccak256(data), expected.as_slice());
    }

    #[test]
    fn test_keccak512() {
        let data = b"hello world";
        let expected = hex::decode("3ee2b40047b8060f68c67242175660f4174d0af5c01d47168ec20ed619b0b7c42181f40aa1046f39e2ef9efc6910782a998e0013d172458957957fac9405b67d").unwrap();
        assert_eq!(super::keccak512(data), expected.as_slice());
    }
}
