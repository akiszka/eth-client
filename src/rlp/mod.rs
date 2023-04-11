//! This module implements the RLP (Recursive Length Prefix) encoding algorithm.
//! https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/
//!
//! RLP is a way to encode arbitrarily nested arrays of binary data, and is used
//! in Ethereum for serializing objects for hashing and signing.

// NOTE: Positive integers are encoded as big-endian binary with no leading zeroes
// (thus making the integer value zero be equivalent to the empty byte array).

// Parts of this implementation and tests are based on an article by @luu-alex
// from https://ethereum.org/en/developers/docs/data-structures-and-encoding/rlp/
// Thanks!

use num_bigint::BigUint;

#[derive(Debug, PartialEq, Clone)]
pub enum RlpEncodable {
    Bytes(Vec<u8>),
    List(Vec<RlpEncodable>),
}

impl RlpEncodable {
    pub fn decode(input: &[u8]) -> Option<RlpEncodable> {
        // this is a recursive function with two termination conditions:
        // 1. the input is Bytes
        // 2. the input is an empty List

        if input.len() == 0 {
            return None;
        }

        let (offset, data_len, is_list) = decode_length(input.to_vec());
        let data = byte_substr(input, offset, data_len);

        if is_list && data_len == 0 {
            return Some(RlpEncodable::List(vec![]));
        } else if !is_list {
            return Some(RlpEncodable::Bytes(data));
        }

        // otherwise, we have a list of RLP encoded items
        // this means that we can decode the first item, and then
        // recursively decode the rest of the list with the remaining input

        let mut result = vec![];
        let mut offset_at = offset;
        let length = offset_at + data_len;

        while offset_at < length {
            let new_data = byte_substr(input, offset_at, length - offset_at);
            if new_data.len() == 0 {
                break;
            }

            let (new_offset, new_data_len, _) = decode_length(new_data.clone());

            if let Some(decoded) = RlpEncodable::decode(&new_data) {
                result.push(decoded);
            } else {
                break;
            }

            offset_at += new_offset + new_data_len;
        }

        Some(RlpEncodable::List(result))
    }

    pub fn encode(&self) -> Vec<u8> {
        match self {
            RlpEncodable::Bytes(bytes) => {
                if bytes.len() == 1 && bytes[0] < 0x80 {
                    bytes.to_owned()
                } else {
                    let mut result = encode_length(bytes.len(), 0x80);
                    result.append(&mut bytes.to_owned());
                    result
                }
            }
            RlpEncodable::List(list) => {
                let mut result = vec![];
                for item in list {
                    result.append(&mut item.encode());
                }
                let mut length_prefix = encode_length(result.len(), 0xc0);
                length_prefix.append(&mut result);
                length_prefix
            }
        }
    }
}

fn encode_length(len: usize, offset: u8) -> Vec<u8> {
    if len < 56 {
        vec![len as u8 + offset]
    } else {
        let mut len_bytes = len.to_be_bytes().to_vec();

        // Remove leading zeroes
        while len_bytes[0] == 0 {
            len_bytes.remove(0);
        }

        let len2 = len_bytes.len() as u8 + offset + 55;

        let mut result = vec![len2];
        result.append(&mut len_bytes);
        result
    }

    // some lengths are unencodable, but we don't care about that here
    // since they are above 256^8 = 2^64 (so larger than usize)
}

// (offset, data_len, is_list)
fn decode_length(input: Vec<u8>) -> (usize, usize, bool) {
    let length = input.len();

    if length == 0 {
        panic!("Invalid RLP: empty input");
    }

    let prefix: usize = input[0].into();

    if prefix <= 0x7f {
        return (0, 1, false);
    } else if prefix <= 0xb7 && length > prefix - 0x80 {
        return (1, (prefix - 0x80), false);
    } else if prefix <= 0xbf
        && length > prefix - 0xb7
        && length > prefix - 0xb7 + usize_byte_substr(&input, 1, prefix - 0xb7)
    {
        let len_of_len = prefix - 0xb7;
        let len = usize_byte_substr(&input, 1, len_of_len);
        return (1 + len_of_len, len, false);
    } else if prefix <= 0xf7 && length > prefix - 0xc0 {
        return (1, (prefix - 0xc0), true);
    } else if prefix <= 0xff
        && length > prefix - 0xf7
        && length > prefix - 0xf7 + usize_byte_substr(&input, 1, prefix - 0xf7)
    {
        let len_of_len = prefix - 0xf7;
        let len = usize_byte_substr(&input, 1, len_of_len);
        return (1 + len_of_len, len, true);
    } else {
        panic!("Invalid RLP: length prefix is non-conformant");
    }
}

fn usize_byte_substr(input: &[u8], offset: usize, length: usize) -> usize {
    let substr = byte_substr(input, offset, length);
    BigUint::from_bytes_be(&substr).try_into().unwrap()
}

fn byte_substr(input: &[u8], offset: usize, length: usize) -> Vec<u8> {
    input[offset..offset + length].to_vec()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn prefix_encode_length() {
        assert_eq!(encode_length(0, 0x80), vec![0x80]);
        assert_eq!(encode_length(55, 0x80), vec![0xb7]);
        assert_eq!(encode_length(56, 0x80), vec![0xb8, 0x38]);
        assert_eq!(encode_length(256, 0x80), vec![0xb9, 0x01, 0x00]);
    }

    #[test]
    fn encode_dog() {
        let dog = RlpEncodable::Bytes("dog".as_bytes().to_vec());
        assert_eq!(dog.encode(), vec![0x83, 0x64, 0x6f, 0x67]);
    }

    #[test]
    fn encode_cat_dog() {
        let cat = RlpEncodable::Bytes("cat".as_bytes().to_vec());
        let dog = RlpEncodable::Bytes("dog".as_bytes().to_vec());
        let cat_dog = RlpEncodable::List(vec![cat, dog]);
        assert_eq!(
            cat_dog.encode(),
            vec![0xc8, 0x83, 0x63, 0x61, 0x74, 0x83, 0x64, 0x6f, 0x67]
        );
    }

    #[test]
    fn encode_empty_string() {
        let empty_string = RlpEncodable::Bytes(vec![]);
        assert_eq!(empty_string.encode(), vec![0x80]);
    }

    #[test]
    fn encode_empty_list() {
        let empty_list = RlpEncodable::List(vec![]);
        assert_eq!(empty_list.encode(), vec![0xc0]);
    }

    #[test]
    fn encode_null() {
        let null = RlpEncodable::Bytes(vec![0x00]);
        assert_eq!(null.encode(), vec![0x00]);
    }

    #[test]
    fn encode_1024() {
        // note: using u32 here would produce vec![0, 0, 4, 0]
        // which is forbidden by the spec
        let num = RlpEncodable::Bytes(1024_u16.to_be_bytes().to_vec());
        assert_eq!(num.encode(), vec![0x82, 0x04, 0x00]);
    }

    #[test]
    fn encode_set_theory_three() {
        let list1 = RlpEncodable::List(vec![]);
        let list2 = RlpEncodable::List(vec![list1.clone()]);
        let list12 = RlpEncodable::List(vec![list1.clone(), list2.clone()]);
        let list = RlpEncodable::List(vec![list1, list2, list12]);

        assert_eq!(
            list.encode(),
            vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]
        );
    }

    #[test]
    fn encode_lorem() {
        let mut lorem: Vec<u8> = "Lorem ipsum dolor sit amet, consectetur adipisicing elit"
            .bytes()
            .collect();
        let lorem_enc = RlpEncodable::Bytes(lorem.clone());

        let mut expected = vec![0xb8, 0x38];
        expected.append(&mut lorem);

        assert_eq!(lorem_enc.encode(), expected)
    }

    #[test]
    fn decode_dog() {
        let dog = RlpEncodable::Bytes("dog".as_bytes().to_vec());
        let encoded = dog.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, dog);
    }

    #[test]
    fn decode_cat_dog() {
        let cat = RlpEncodable::Bytes("cat".as_bytes().to_vec());
        let dog = RlpEncodable::Bytes("dog".as_bytes().to_vec());
        let cat_dog = RlpEncodable::List(vec![cat, dog]);
        let encoded = cat_dog.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, cat_dog);
    }

    #[test]
    fn decode_empty_string() {
        let empty_string = RlpEncodable::Bytes(vec![]);
        let encoded = empty_string.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, empty_string);
    }

    #[test]
    fn decode_empty_list() {
        let empty_list = RlpEncodable::List(vec![]);
        let encoded = empty_list.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, empty_list);
    }

    #[test]
    fn decode_null() {
        let null = RlpEncodable::Bytes(vec![0x00]);
        let encoded = null.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, null);
    }

    #[test]
    fn decode_1024() {
        let num = RlpEncodable::Bytes(1024_u16.to_be_bytes().to_vec());
        let encoded = num.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, num);
    }

    #[test]
    fn decode_set_theory_three() {
        let list1 = RlpEncodable::List(vec![]);
        let list2 = RlpEncodable::List(vec![list1.clone()]);
        let list12 = RlpEncodable::List(vec![list1.clone(), list2.clone()]);
        let list = RlpEncodable::List(vec![list1, list2, list12]);

        let encoded = list.encode();
        assert_eq!(
            encoded,
            vec![0xc7, 0xc0, 0xc1, 0xc0, 0xc3, 0xc0, 0xc1, 0xc0]
        );

        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, list);
    }

    #[test]
    fn decode_lorem() {
        let lorem: Vec<u8> = "Lorem ipsum dolor sit amet, consectetur adipisicing elit"
            .bytes()
            .collect();
        let lorem_enc = RlpEncodable::Bytes(lorem.clone());

        let encoded = lorem_enc.encode();
        let decoded = RlpEncodable::decode(&encoded).unwrap();
        assert_eq!(decoded, lorem_enc);
    }
}
