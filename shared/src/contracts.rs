use ethers::{
    abi::{Bytes, Token, Tokenize, Word},
    utils::keccak256,
};
use std::convert::TryInto;

pub fn function_identifier<B: AsRef<[u8]>>(sig: B) -> [u8; 4] {
    keccak256(sig)[..4].try_into().unwrap()
}

pub fn encode_args<T: Tokenize>(args: T) -> Bytes {
    encode_tokens(&args.into_tokens())
}

pub fn encode_call<T: Tokenize>(sig: &str, args: T) -> Vec<u8> {
    let id = if sig.starts_with("0x") {
        crate::util::decode_hex(sig).unwrap()
    } else {
        function_identifier(sig).to_vec()
    };
    let args = encode_args(args);
    id.into_iter().chain(args).collect()
}

fn encode_tokens(tokens: &[Token]) -> Bytes {
    let mediates = &tokens.iter().map(encode_token).collect::<Vec<_>>();

    encode_head_tail(mediates)
        .iter()
        .flat_map(|word| word.to_vec())
        .collect()
}

fn pad_u32(value: u32) -> Word {
    let mut padded = [0u8; 32];
    padded[28..32].copy_from_slice(&value.to_be_bytes());
    padded
}

fn pad_bytes(bytes: &[u8]) -> Vec<Word> {
    let mut result = vec![pad_u32(bytes.len() as u32)];
    result.extend(pad_fixed_bytes(bytes));
    result
}

fn pad_fixed_bytes(bytes: &[u8]) -> Vec<Word> {
    let len = (bytes.len() + 31) / 32;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let mut padded = [0u8; 32];

        let to_copy = match i == len - 1 {
            false => 32,
            true => match bytes.len() % 32 {
                0 => 32,
                x => x,
            },
        };

        let offset = 32 * i;
        padded[..to_copy].copy_from_slice(&bytes[offset..offset + to_copy]);
        result.push(padded);
    }

    result
}

#[derive(Debug)]
enum Mediate {
    Raw(Vec<Word>),
    Prefixed(Vec<Word>),
    PrefixedArray(Vec<Mediate>),
    PrefixedArrayWithLength(Vec<Mediate>),
    RawTuple(Vec<Mediate>),
    PrefixedTuple(Vec<Mediate>),
}

impl Mediate {
    fn head_len(&self) -> u32 {
        match *self {
            Mediate::Raw(ref raw) => 32 * raw.len() as u32,
            Mediate::RawTuple(ref mediates) => {
                mediates.iter().map(|mediate| mediate.head_len()).sum()
            }
            Mediate::Prefixed(_)
            | Mediate::PrefixedArray(_)
            | Mediate::PrefixedArrayWithLength(_)
            | Mediate::PrefixedTuple(_) => 32,
        }
    }

    fn tail_len(&self) -> u32 {
        match *self {
            Mediate::Raw(_) | Mediate::RawTuple(_) => 0,
            Mediate::Prefixed(ref pre) => pre.len() as u32 * 32,
            Mediate::PrefixedArray(ref mediates) => mediates
                .iter()
                .fold(0, |acc, m| acc + m.head_len() + m.tail_len()),
            Mediate::PrefixedArrayWithLength(ref mediates) => mediates
                .iter()
                .fold(32, |acc, m| acc + m.head_len() + m.tail_len()),
            Mediate::PrefixedTuple(ref mediates) => mediates
                .iter()
                .fold(0, |acc, m| acc + m.head_len() + m.tail_len()),
        }
    }

    fn head(&self, suffix_offset: u32) -> Vec<Word> {
        match *self {
            Mediate::Raw(ref raw) => raw.clone(),
            Mediate::RawTuple(ref raw) => raw
                .iter()
                .map(|mediate| mediate.head(0))
                .flatten()
                .collect(),
            Mediate::Prefixed(_)
            | Mediate::PrefixedArray(_)
            | Mediate::PrefixedArrayWithLength(_)
            | Mediate::PrefixedTuple(_) => vec![pad_u32(suffix_offset)],
        }
    }

    fn tail(&self) -> Vec<Word> {
        match *self {
            Mediate::Raw(_) | Mediate::RawTuple(_) => vec![],
            Mediate::PrefixedTuple(ref mediates) => encode_head_tail(mediates),
            Mediate::Prefixed(ref raw) => raw.clone(),
            Mediate::PrefixedArray(ref mediates) => encode_head_tail(mediates),
            Mediate::PrefixedArrayWithLength(ref mediates) => {
                // + 32 added to offset represents len of the array prepanded to tail
                let mut result = vec![pad_u32(mediates.len() as u32)];

                let head_tail = encode_head_tail(mediates);

                result.extend(head_tail);
                result
            }
        }
    }
}

fn encode_head_tail(mediates: &[Mediate]) -> Vec<Word> {
    let heads_len = mediates.iter().fold(0, |acc, m| acc + m.head_len());

    let (mut result, len) = mediates.iter().fold(
        (Vec::with_capacity(heads_len as usize), heads_len),
        |(mut acc, offset), m| {
            acc.extend(m.head(offset));
            (acc, offset + m.tail_len())
        },
    );

    let tails = mediates.iter().fold(
        Vec::with_capacity((len - heads_len) as usize),
        |mut acc, m| {
            acc.extend(m.tail());
            acc
        },
    );

    result.extend(tails);
    result
}

fn encode_token(token: &Token) -> Mediate {
    match *token {
        Token::Address(ref address) => {
            let mut padded = [0u8; 32];
            padded[12..].copy_from_slice(address.as_ref());
            Mediate::Raw(vec![padded])
        }
        Token::Bytes(ref bytes) => Mediate::Prefixed(pad_bytes(bytes)),
        Token::String(ref s) => Mediate::Prefixed(pad_bytes(s.as_bytes())),
        Token::FixedBytes(ref bytes) => Mediate::Raw(pad_fixed_bytes(bytes)),
        Token::Int(int) => Mediate::Raw(vec![int.into()]),
        Token::Uint(uint) => Mediate::Raw(vec![uint.into()]),
        Token::Bool(b) => {
            let mut value = [0u8; 32];
            if b {
                value[31] = 1;
            }
            Mediate::Raw(vec![value])
        }
        Token::Array(ref tokens) => {
            let mediates = tokens.iter().map(encode_token).collect();

            Mediate::PrefixedArrayWithLength(mediates)
        }
        Token::FixedArray(ref tokens) => {
            let mediates = tokens.iter().map(encode_token).collect();

            if token.is_dynamic() {
                Mediate::PrefixedArray(mediates)
            } else {
                Mediate::Raw(encode_head_tail(&mediates))
            }
        }
        Token::Tuple(ref tokens) if token.is_dynamic() => {
            let mediates = tokens.iter().map(encode_token).collect();

            Mediate::PrefixedTuple(mediates)
        }
        Token::Tuple(ref tokens) => {
            let mediates = tokens.iter().map(encode_token).collect();

            Mediate::RawTuple(mediates)
        }
    }
}

#[cfg(test)]
mod test {
    use ethers::types::U256;

    #[test]
    fn function_identifier() {
        assert_eq!(super::function_identifier("atomicMatch_(address[14],uint256[18],uint8[8],bytes,bytes,bytes,bytes,bytes,bytes,uint8[2],bytes32[5])"), [171u8, 131, 75, 171]);
    }

    #[test]
    fn calldata() {
        let func_sig = super::function_identifier("adopt(uint256)").to_vec();
        let func_args = super::encode_args([U256::from(2)]);
        let func_calldata = func_sig
            .into_iter()
            .chain(func_args.into_iter())
            .collect::<Vec<_>>();
        assert_eq!(
            hex::encode(func_calldata),
            "8588b2c50000000000000000000000000000000000000000000000000000000000000002"
        )
    }
}
