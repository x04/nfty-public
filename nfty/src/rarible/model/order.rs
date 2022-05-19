use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "maker")]
    pub maker: String,

    #[serde(rename = "make")]
    pub make: Asset,

    #[serde(rename = "makePriceUsd")]
    pub make_price_usd: f64,

    #[serde(rename = "take")]
    pub take: Asset,

    #[serde(rename = "takeCurrency")]
    pub take_currency: TakeCurrency,

    #[serde(rename = "type")]
    pub orders_type: String,

    #[serde(rename = "fill")]
    pub fill: String,

    #[serde(rename = "makeStock")]
    pub make_stock: String,

    #[serde(rename = "cancelled")]
    pub cancelled: bool,

    #[serde(rename = "salt")]
    pub salt: String,

    #[serde(rename = "data")]
    pub data: Data,

    #[serde(rename = "signature")]
    pub signature: String,

    #[serde(rename = "createdAt")]
    pub created_at: String,

    #[serde(rename = "lastUpdateAt")]
    pub last_update_at: String,

    #[serde(rename = "pending")]
    pub pending: Vec<Option<serde_json::Value>>,

    #[serde(rename = "makeValueCurrency")]
    pub make_value_currency: i64,

    #[serde(rename = "takeValueCurrency")]
    pub take_value_currency: f64,

    #[serde(rename = "sold")]
    pub sold: String,

    #[serde(rename = "sellPrice")]
    pub sell_price: f64,

    #[serde(rename = "buyPrice")]
    pub buy_price: f64,

    #[serde(rename = "sellPriceEth")]
    pub sell_price_eth: f64,

    #[serde(rename = "active")]
    pub active: bool,

    #[serde(rename = "makeToken")]
    pub make_token: String,

    #[serde(rename = "makeTokenId")]
    pub make_token_id: String,

    #[serde(rename = "takeToken")]
    pub take_token: String,

    #[serde(rename = "sellOrder")]
    pub sell_order: bool,

    #[serde(rename = "takeTokenId")]
    pub take_token_id: String,

    #[serde(rename = "offer")]
    pub offer: bool,

    #[serde(rename = "completed")]
    pub completed: bool,

    #[serde(rename = "itemId")]
    pub item_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "dataType")]
    pub data_type: String,

    #[serde(rename = "payouts")]
    pub payouts: Vec<Option<serde_json::Value>>,

    #[serde(rename = "originFees")]
    pub origin_fees: Vec<OriginFee>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OriginFee {
    #[serde(rename = "account")]
    pub account: String,

    #[serde(rename = "value")]
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "tokenId")]
    pub token_id: String,

    #[serde(rename = "assetType")]
    pub asset_type: String,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TakeCurrency {
    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "blockchain")]
    pub blockchain: String,

    #[serde(rename = "decimals")]
    pub decimals: i64,

    #[serde(rename = "rate")]
    pub rate: i64,

    #[serde(rename = "allowed")]
    pub allowed: bool,

    #[serde(rename = "useUniswap")]
    pub use_uniswap: bool,

    #[serde(rename = "order")]
    pub order: i64,
}
