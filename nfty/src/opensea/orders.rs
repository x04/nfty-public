use super::shared_types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Orders {
    #[serde(rename = "count")]
    pub count: Option<i64>,

    #[serde(rename = "orders")]
    pub orders: Vec<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "asset")]
    pub asset: Option<OrderAsset>,

    #[serde(rename = "asset_bundle")]
    pub asset_bundle: Option<serde_json::Value>,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "closing_date")]
    pub closing_date: Option<serde_json::Value>,

    #[serde(rename = "closing_extendable")]
    pub closing_extendable: Option<bool>,

    #[serde(rename = "expiration_time")]
    pub expiration_time: u64,

    #[serde(rename = "listing_time")]
    pub listing_time: u64,

    #[serde(rename = "order_hash")]
    pub order_hash: Option<String>,

    #[serde(rename = "metadata")]
    pub metadata: Option<Metadata>,

    #[serde(rename = "exchange")]
    pub exchange: String,

    #[serde(rename = "maker")]
    pub maker: FeeRecipient,

    #[serde(rename = "taker")]
    pub taker: FeeRecipient,

    #[serde(rename = "current_price")]
    pub current_price: Option<String>,

    #[serde(rename = "current_bounty")]
    pub current_bounty: Option<String>,

    #[serde(rename = "bounty_multiple")]
    pub bounty_multiple: Option<String>,

    #[serde(rename = "maker_relayer_fee")]
    pub maker_relayer_fee: String,

    #[serde(rename = "taker_relayer_fee")]
    pub taker_relayer_fee: String,

    #[serde(rename = "maker_protocol_fee")]
    pub maker_protocol_fee: String,

    #[serde(rename = "taker_protocol_fee")]
    pub taker_protocol_fee: String,

    #[serde(rename = "maker_referrer_fee")]
    pub maker_referrer_fee: Option<String>,

    #[serde(rename = "fee_recipient")]
    pub fee_recipient: FeeRecipient,

    #[serde(rename = "fee_method")]
    pub fee_method: u8,

    #[serde(rename = "side")]
    pub side: u8,

    #[serde(rename = "sale_kind")]
    pub sale_kind: u8,

    #[serde(rename = "target")]
    pub target: String,

    #[serde(rename = "how_to_call")]
    pub how_to_call: u8,

    #[serde(rename = "calldata")]
    pub calldata: String,

    #[serde(rename = "replacement_pattern")]
    pub replacement_pattern: String,

    #[serde(rename = "static_target")]
    pub static_target: String,

    #[serde(rename = "static_extradata")]
    pub static_extradata: String,

    #[serde(rename = "payment_token")]
    pub payment_token: String,

    #[serde(rename = "payment_token_contract")]
    pub payment_token_contract: Option<PaymentTokenContract>,

    #[serde(rename = "base_price")]
    pub base_price: String,

    #[serde(rename = "extra")]
    pub extra: String,

    #[serde(rename = "quantity")]
    pub quantity: String,

    #[serde(rename = "salt")]
    pub salt: String,

    #[serde(rename = "v")]
    pub v: u8,

    #[serde(rename = "r")]
    pub r: String,

    #[serde(rename = "s")]
    pub s: String,

    #[serde(rename = "approved_on_chain")]
    pub approved_on_chain: Option<bool>,

    #[serde(rename = "cancelled")]
    pub cancelled: Option<bool>,

    #[serde(rename = "finalized")]
    pub finalized: Option<bool>,

    #[serde(rename = "marked_invalid")]
    pub marked_invalid: Option<bool>,

    #[serde(rename = "prefixed_hash")]
    pub prefixed_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderAsset {
    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "token_id")]
    pub token_id: Option<String>,

    #[serde(rename = "num_sales")]
    pub num_sales: Option<i64>,

    #[serde(rename = "background_color")]
    pub background_color: Option<serde_json::Value>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "image_preview_url")]
    pub image_preview_url: Option<String>,

    #[serde(rename = "image_thumbnail_url")]
    pub image_thumbnail_url: Option<String>,

    #[serde(rename = "image_original_url")]
    pub image_original_url: Option<String>,

    #[serde(rename = "animation_url")]
    pub animation_url: Option<serde_json::Value>,

    #[serde(rename = "animation_original_url")]
    pub animation_original_url: Option<serde_json::Value>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "description")]
    pub description: Option<serde_json::Value>,

    #[serde(rename = "external_link")]
    pub external_link: Option<String>,

    #[serde(rename = "asset_contract")]
    pub asset_contract: Option<AssetContract>,

    #[serde(rename = "permalink")]
    pub permalink: Option<String>,

    #[serde(rename = "collection")]
    pub collection: Option<Collection>,

    #[serde(rename = "decimals")]
    pub decimals: Option<i64>,

    #[serde(rename = "token_metadata")]
    pub token_metadata: Option<String>,

    #[serde(rename = "owner")]
    pub owner: Option<FeeRecipient>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeeRecipient {
    #[serde(rename = "user")]
    pub user: Option<User>,

    #[serde(rename = "profile_img_url")]
    pub profile_img_url: Option<String>,

    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "config")]
    pub config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "asset")]
    pub asset: Option<MetadataAsset>,

    #[serde(rename = "schema")]
    pub schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataAsset {
    #[serde(rename = "id")]
    pub id: Option<String>,

    #[serde(rename = "address")]
    pub address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentTokenContract {
    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "address")]
    pub address: Option<String>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "decimals")]
    pub decimals: Option<i64>,

    #[serde(rename = "eth_price")]
    pub eth_price: Option<String>,

    #[serde(rename = "usd_price")]
    pub usd_price: Option<String>,
}
