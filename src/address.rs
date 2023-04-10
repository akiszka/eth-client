use std::fmt::Display;

use num_bigint::{BigInt, BigUint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address(BigUint);

impl From<BigUint> for Address {
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

impl From<BigInt> for Address {
    fn from(value: BigInt) -> Self {
        let (_, value) = value.into_parts();
        Self(value)
    }
}

impl From<Address> for String {
    fn from(value: Address) -> Self {
        // TODO: implement checksum
        let mut bytes = value.0.to_bytes_le();

        while bytes.len() < 20 {
            bytes.push(0);
        }

        format!("0x{}", hex::encode(bytes.into_iter().rev().collect::<Vec<_>>()))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", String::from(self.clone()))
    }
}
