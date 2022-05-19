use crate::config::MintArgument;
use ethers::{
    abi::{Token as AbiToken, Tokenizable},
    prelude::*,
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use toml::Value;

use crate::util;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Token {
    /// Address.
    ///
    /// solidity name: address
    /// Encoded to left padded [0u8; 32].
    Address,
    /// Vector of bytes with known size.
    ///
    /// solidity name eg.: bytes8, bytes32, bytes64, bytes1024
    /// Encoded to right padded [0u8; ((N + 31) / 32) * 32].
    FixedBytes,
    /// Vector of bytes of unknown size.
    ///
    /// solidity name: bytes
    /// Encoded in two parts.
    /// Init part: offset of 'closing part`.
    /// Closing part: encoded length followed by encoded right padded bytes.
    Bytes,
    /// Signed integer.
    ///
    /// solidity name: int
    Int,
    /// Unisnged integer.
    ///
    /// solidity name: uint
    Uint,
    /// Boolean value.
    ///
    /// solidity name: bool
    /// Encoded as left padded [0u8; 32], where last bit represents boolean value.
    Bool,
    /// String.
    ///
    /// solidity name: string
    /// Encoded in the same way as bytes. Must be utf8 compliant.
    String,
    /// Array with known size.
    ///
    /// solidity name eg.: int[3], bool[3], address[][8]
    /// Encoding of array is equal to encoding of consecutive elements of array.
    FixedArray,
    /// Array of params with unknown size.
    ///
    /// solidity name eg. int[], bool[], address[5][]
    Array,
    /// Tuple of params of variable types.
    ///
    /// solidity name: tuple
    Tuple,
}

impl Token {
    pub fn to_token(&self, value: &Value) -> Result<AbiToken, crate::Error> {
        Ok(match self {
            Self::Uint => match value {
                Value::Integer(i) if *i >= 0 => U256::from(*i).into_token(),
                Value::Float(i) if *i >= 0. => U256::from(*i as u128).into_token(),
                Value::String(s) => U256::from_str(s)?.into_token(),
                _ => unimplemented!(),
            },
            Self::String => match value {
                Value::String(s) => AbiToken::String(s.to_string()),
                _ => unimplemented!(),
            },
            Self::Bool => match value {
                Value::Boolean(b) => AbiToken::Bool(*b),
                _ => unimplemented!(),
            },
            Self::Address => match value {
                Value::String(s) => AbiToken::Address(Address::from_str(s)?),
                _ => unimplemented!(),
            },
            Self::FixedBytes => match value {
                Value::String(s) => AbiToken::FixedBytes(util::decode_hex(s)?),
                _ => unimplemented!(),
            },
            Self::Bytes => match value {
                Value::String(s) => AbiToken::Bytes(util::decode_hex(s)?),
                _ => unimplemented!(),
            },
            Self::Tuple => match value {
                Value::Array(a) => AbiToken::Tuple(
                    a.to_vec()
                        .into_iter()
                        .map(|x| {
                            toml::from_slice::<MintArgument>(&toml::to_vec(&x).unwrap()).unwrap()
                        })
                        .map(|x| x.r#type.to_token(&x.value).unwrap())
                        .collect::<Vec<AbiToken>>(),
                ),
                _ => unimplemented!(),
            },
            Self::Array => match value {
                Value::Array(a) => AbiToken::Array(
                    a.to_vec()
                        .into_iter()
                        .map(|x| {
                            toml::from_slice::<MintArgument>(&toml::to_vec(&x).unwrap()).unwrap()
                        })
                        .map(|x| x.r#type.to_token(&x.value).unwrap())
                        .collect::<Vec<AbiToken>>(),
                ),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        })
    }
}
