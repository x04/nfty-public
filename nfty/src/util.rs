use crate::{
    model::{AtomicMatchArgs, AtomicOrder, AtomicSig, OldOrder},
    opensea::Order,
};
use ethers::{
    abi::{Token, Uint},
    prelude::{transaction::eip2718::TypedTransaction, *},
};
use log::*;
use rand::{thread_rng, Rng};
use shared::{config::Config as NftyConfig, contracts, util};
use std::{convert::TryInto, str::FromStr};

const NFTY_TAG: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // nfty
    110, 102, 116, 121,
];
const OPENSEA_CONTRACT: &str = "0x7be8076f4ea4a4ad08075c2508e481d6c946d12b";
pub const NULL_ADDR: Address = H160([0u8; 20]);
pub const SALE_SIDE_BUY: u8 = 0;

// TODO: refactor, too lazy rn
fn shift_mask(mask: &[u8]) -> Vec<u8> {
    assert!(mask.len() > 32);

    let mut stop_idx = 0;
    let mut new_mask = vec![0; mask.len()];
    for i in 0..mask.len() {
        if mask[i] != 0xff {
            continue;
        } else if stop_idx == 0 {
            stop_idx = i + 32
        } else if stop_idx != 0 && i == stop_idx {
            break;
        }
        let new_value = mask[i - 32];
        new_mask[i - 32] = mask[i];
        new_mask[i] = new_value;
    }
    new_mask
}

fn censor_calldata(data: &mut [u8], mask: &[u8]) {
    assert_eq!(data.len(), mask.len());

    data.iter_mut().zip(mask.iter()).for_each(|(x, &y)| {
        if y == 0xff {
            *x = 0;
        }
    });
}

// TODO: refactor this too
fn insert_buyer_address(data: &mut [u8], mask: &[u8], addr: Address) {
    assert_eq!(data.len(), mask.len());

    let mut addr_padded = vec![0u8; 12];
    addr_padded.append(&mut addr.0.to_vec());
    for i in 0..data.len() {
        if i == 0 {
            continue;
        }

        if mask[i] == 0xff && mask[i - 1] == 00 {
            for (i2, &v) in addr_padded.iter().enumerate() {
                data[i + i2] = v
            }
            break;
        }
    }
}

pub async fn order_to_tx<M: 'static + Middleware, S: 'static + Signer>(
    config: &NftyConfig,
    provider: &SignerMiddleware<M, S>,
    our_addr: Address,
    order: &OldOrder,
    gas_fee: U256,
    priority_fee: U256,
    nonce: U256,
) -> Result<TypedTransaction, shared::Error> {
    // TODO: make const
    let opensea_address = Address::from_str(OPENSEA_CONTRACT)?;

    let target = Address::from_str(&order.target)?;
    let exchange = Address::from_str(&order.exchange)?;
    let maker = Address::from_str(&order.maker.address)?;
    let taker = Address::from_str(&order.taker.address)?;
    let payment_token = Address::from_str(&order.payment_token)?;
    let fee_recipient = Address::from_str(&order.fee_recipient.address)?;
    let static_target = Address::from_str(&order.static_target)?;

    let base_price = U256::from_dec_str(&order.base_price)?;
    let extra = U256::from_dec_str(&order.extra)?;
    let maker_relayer_fee = U256::from_dec_str(&order.maker_relayer_fee)?;
    let taker_relayer_fee = U256::from_dec_str(&order.taker_relayer_fee)?;
    let maker_protocol_fee = U256::from_dec_str(&order.maker_protocol_fee)?;
    let taker_protocol_fee = U256::from_dec_str(&order.taker_protocol_fee)?;
    let fee_method = order.fee_method;
    let sale_kind = order.sale_kind;
    let how_to_call = order.how_to_call;

    let static_extra_data = util::decode_hex(&order.static_extradata)?;
    let sell_replacement = util::decode_hex(&order.replacement_pattern)?;
    let buy_replacement = shift_mask(&sell_replacement);

    let sell_calldata = util::decode_hex(&order.calldata)?;
    let mut buy_calldata = sell_calldata.clone();
    censor_calldata(&mut buy_calldata, &buy_replacement);
    insert_buyer_address(&mut buy_calldata, &sell_replacement, our_addr);

    let args = AtomicMatchArgs {
        buy: AtomicOrder {
            exchange,
            maker: our_addr,
            taker: NULL_ADDR,
            maker_relayer_fee,
            taker_relayer_fee,
            maker_protocol_fee,
            taker_protocol_fee,
            fee_recipient: NULL_ADDR,
            fee_method,
            side: SALE_SIDE_BUY,
            sale_kind,
            target,
            how_to_call,
            calldata: buy_calldata,
            replacement_pattern: buy_replacement,
            static_target: NULL_ADDR,
            static_extra_data: Vec::new(),
            payment_token,
            base_price,
            extra,
            listing_time: U256::from(0),
            expiration_time: U256::from(0),
            salt: U256::from(thread_rng().gen::<[u8; 32]>()),
        },
        buy_sig: AtomicSig {
            v: 0,
            r: NFTY_TAG.to_vec(),
            s: vec![0; 32],
        },
        sell: AtomicOrder {
            exchange,
            maker,
            taker,
            maker_relayer_fee,
            taker_relayer_fee,
            maker_protocol_fee,
            taker_protocol_fee,
            fee_recipient,
            fee_method,
            side: order.side,
            sale_kind,
            target,
            how_to_call,
            calldata: sell_calldata,
            replacement_pattern: sell_replacement,
            static_target,
            static_extra_data,
            payment_token,
            base_price,
            extra,
            listing_time: U256::from(order.listing_time),
            expiration_time: U256::from(order.expiration_time),
            salt: U256::from_dec_str(&order.salt)?,
        },
        sell_sig: AtomicSig {
            v: order.v,
            r: util::decode_hex(&order.r)?,
            s: util::decode_hex(&order.s)?,
        },
    };

    let calldata =
        contracts::encode_call(
            "atomicMatch_(address[14],uint256[18],uint8[8],bytes,bytes,bytes,bytes,bytes,bytes,uint8[2],bytes32[5])",
            (
                [
                    args.buy.exchange,
                    args.buy.maker,
                    args.buy.taker,
                    args.buy.fee_recipient,
                    args.buy.target,
                    args.buy.static_target,
                    args.buy.payment_token,
                    args.sell.exchange,
                    args.sell.maker,
                    args.sell.taker,
                    args.sell.fee_recipient,
                    args.sell.target,
                    args.sell.static_target,
                    args.sell.payment_token,
                ],
                [
                    args.buy.maker_relayer_fee,
                    args.buy.taker_relayer_fee,
                    args.buy.maker_protocol_fee,
                    args.buy.taker_protocol_fee,
                    args.buy.base_price,
                    args.buy.extra,
                    args.buy.listing_time,
                    args.buy.expiration_time,
                    args.buy.salt,
                    args.sell.maker_relayer_fee,
                    args.sell.taker_relayer_fee,
                    args.sell.maker_protocol_fee,
                    args.sell.taker_protocol_fee,
                    args.sell.base_price,
                    args.sell.extra,
                    args.sell.listing_time,
                    args.sell.expiration_time,
                    args.sell.salt,
                ],
                [
                    Token::Uint(Uint::from(args.buy.fee_method)),
                    Token::Uint(Uint::from(0)),
                    Token::Uint(Uint::from(args.buy.sale_kind)),
                    Token::Uint(Uint::from(args.buy.how_to_call)),
                    Token::Uint(Uint::from(args.sell.fee_method)),
                    Token::Uint(Uint::from(1)),
                    Token::Uint(Uint::from(args.sell.sale_kind)),
                    Token::Uint(Uint::from(args.sell.how_to_call)),
                ],
                args.buy.calldata,
                args.sell.calldata,
                args.buy.replacement_pattern,
                args.sell.replacement_pattern,
                args.buy.static_extra_data,
                args.sell.static_extra_data,
                [
                    Uint::from(args.buy_sig.v),
                    Uint::from(args.sell_sig.v),
                ],
                [
                    args.buy_sig.r.as_slice().try_into()?,
                    args.buy_sig.s.as_slice().try_into()?,
                    args.sell_sig.r.as_slice().try_into()?,
                    args.sell_sig.s.as_slice().try_into()?,
                    [0u8; 32],
                ],
            )
        );

    let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
        from: Some(our_addr),
        to: Some(opensea_address.into()),
        value: Some(base_price),
        data: Some(calldata.into()),
        nonce: Some(nonce),
        max_priority_fee_per_gas: Some(priority_fee),
        max_fee_per_gas: Some(gas_fee),
        ..Default::default()
    });

    let opensea_config = config.opensea.as_ref().expect("expected OpenSea config");
    let gas_amount = if opensea_config.estimate_gas {
        match provider.estimate_gas(&tx).await {
            Ok(amount) => U256::from((amount.as_u64() as f64 * 1.2) as u64),
            Err(e) => {
                warn!("error estimating gas amount: {}", e);
                U256::from(opensea_config.gas_limit)
            }
        }
    } else {
        U256::from(opensea_config.gas_limit)
    };
    tx.set_gas(gas_amount);

    Ok(tx)
}

pub async fn new_order_to_tx<M: 'static + Middleware, S: 'static + Signer>(
    config: &NftyConfig,
    provider: &SignerMiddleware<M, S>,
    our_addr: Address,
    order: &Order,
    gas_fee: U256,
    priority_fee: U256,
    nonce: U256,
) -> Result<TypedTransaction, shared::Error> {
    // TODO: make const
    let opensea_address = Address::from_str(OPENSEA_CONTRACT)?;

    let target = Address::from_str(&order.target)?;
    let exchange = Address::from_str(&order.exchange)?;
    let maker = Address::from_str(&order.maker.address)?;
    let taker = Address::from_str(&order.taker.address)?;
    let payment_token = Address::from_str(&order.payment_token)?;
    let fee_recipient = Address::from_str(&order.fee_recipient.address)?;
    let static_target = Address::from_str(&order.static_target)?;

    let base_price = U256::from_dec_str(&order.base_price)?;
    let extra = U256::from_dec_str(&order.extra)?;
    let maker_relayer_fee = U256::from_dec_str(&order.maker_relayer_fee)?;
    let taker_relayer_fee = U256::from_dec_str(&order.taker_relayer_fee)?;
    let maker_protocol_fee = U256::from_dec_str(&order.maker_protocol_fee)?;
    let taker_protocol_fee = U256::from_dec_str(&order.taker_protocol_fee)?;
    let fee_method = order.fee_method;
    let sale_kind = order.sale_kind;
    let how_to_call = order.how_to_call;

    let static_extra_data = util::decode_hex(&order.static_extradata)?;
    let sell_replacement = util::decode_hex(&order.replacement_pattern)?;
    let buy_replacement = shift_mask(&sell_replacement);

    let sell_calldata = util::decode_hex(&order.calldata)?;
    let mut buy_calldata = sell_calldata.clone();
    censor_calldata(&mut buy_calldata, &buy_replacement);
    insert_buyer_address(&mut buy_calldata, &sell_replacement, our_addr);

    let args = AtomicMatchArgs {
        buy: AtomicOrder {
            exchange,
            maker: our_addr,
            taker: NULL_ADDR,
            maker_relayer_fee,
            taker_relayer_fee,
            maker_protocol_fee,
            taker_protocol_fee,
            fee_recipient: NULL_ADDR,
            fee_method,
            side: SALE_SIDE_BUY,
            sale_kind,
            target,
            how_to_call,
            calldata: buy_calldata,
            replacement_pattern: buy_replacement,
            static_target: NULL_ADDR,
            static_extra_data: Vec::new(),
            payment_token,
            base_price,
            extra,
            listing_time: U256::from(0),
            expiration_time: U256::from(0),
            salt: U256::from(thread_rng().gen::<[u8; 32]>()),
        },
        buy_sig: AtomicSig {
            v: 0,
            r: NFTY_TAG.to_vec(),
            s: vec![0; 32],
        },
        sell: AtomicOrder {
            exchange,
            maker,
            taker,
            maker_relayer_fee,
            taker_relayer_fee,
            maker_protocol_fee,
            taker_protocol_fee,
            fee_recipient,
            fee_method,
            side: order.side,
            sale_kind,
            target,
            how_to_call,
            calldata: sell_calldata,
            replacement_pattern: sell_replacement,
            static_target,
            static_extra_data,
            payment_token,
            base_price,
            extra,
            listing_time: U256::from(order.listing_time),
            expiration_time: U256::from(order.expiration_time),
            salt: U256::from_dec_str(&order.salt)?,
        },
        sell_sig: AtomicSig {
            v: order.v,
            r: util::decode_hex(&order.r)?,
            s: util::decode_hex(&order.s)?,
        },
    };

    let calldata =
        contracts::encode_call(
            "atomicMatch_(address[14],uint256[18],uint8[8],bytes,bytes,bytes,bytes,bytes,bytes,uint8[2],bytes32[5])",
            (
                [
                    args.buy.exchange,
                    args.buy.maker,
                    args.buy.taker,
                    args.buy.fee_recipient,
                    args.buy.target,
                    args.buy.static_target,
                    args.buy.payment_token,
                    args.sell.exchange,
                    args.sell.maker,
                    args.sell.taker,
                    args.sell.fee_recipient,
                    args.sell.target,
                    args.sell.static_target,
                    args.sell.payment_token,
                ],
                [
                    args.buy.maker_relayer_fee,
                    args.buy.taker_relayer_fee,
                    args.buy.maker_protocol_fee,
                    args.buy.taker_protocol_fee,
                    args.buy.base_price,
                    args.buy.extra,
                    args.buy.listing_time,
                    args.buy.expiration_time,
                    args.buy.salt,
                    args.sell.maker_relayer_fee,
                    args.sell.taker_relayer_fee,
                    args.sell.maker_protocol_fee,
                    args.sell.taker_protocol_fee,
                    args.sell.base_price,
                    args.sell.extra,
                    args.sell.listing_time,
                    args.sell.expiration_time,
                    args.sell.salt,
                ],
                [
                    Token::Uint(Uint::from(args.buy.fee_method)),
                    Token::Uint(Uint::from(0)),
                    Token::Uint(Uint::from(args.buy.sale_kind)),
                    Token::Uint(Uint::from(args.buy.how_to_call)),
                    Token::Uint(Uint::from(args.sell.fee_method)),
                    Token::Uint(Uint::from(1)),
                    Token::Uint(Uint::from(args.sell.sale_kind)),
                    Token::Uint(Uint::from(args.sell.how_to_call)),
                ],
                args.buy.calldata,
                args.sell.calldata,
                args.buy.replacement_pattern,
                args.sell.replacement_pattern,
                args.buy.static_extra_data,
                args.sell.static_extra_data,
                [
                    Uint::from(args.buy_sig.v),
                    Uint::from(args.sell_sig.v),
                ],
                [
                    args.buy_sig.r.as_slice().try_into()?,
                    args.buy_sig.s.as_slice().try_into()?,
                    args.sell_sig.r.as_slice().try_into()?,
                    args.sell_sig.s.as_slice().try_into()?,
                    [0u8; 32],
                ],
            )
        );

    let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest {
        from: Some(our_addr),
        to: Some(opensea_address.into()),
        value: Some(base_price),
        data: Some(calldata.into()),
        nonce: Some(nonce),
        max_priority_fee_per_gas: Some(priority_fee),
        max_fee_per_gas: Some(gas_fee),
        ..Default::default()
    });

    let opensea_config = config.opensea.as_ref().expect("expected OpenSea config");
    let gas_amount = if opensea_config.estimate_gas {
        match provider.estimate_gas(&tx).await {
            Ok(amount) => U256::from((amount.as_u64() as f64 * 1.2) as u64),
            Err(e) => {
                warn!("error estimating gas amount: {}", e);
                U256::from(opensea_config.gas_limit)
            }
        }
    } else {
        U256::from(opensea_config.gas_limit)
    };
    tx.set_gas(gas_amount);

    Ok(tx)
}
