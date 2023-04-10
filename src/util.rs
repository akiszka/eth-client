use hex::FromHexError;
use num_bigint::BigInt;

pub fn bigint_from_hex(hex: &str) -> Result<BigInt, FromHexError> {
    let bytes = hex::decode(hex)?;
    Ok(BigInt::from_bytes_be(num_bigint::Sign::Plus, &bytes))
}
