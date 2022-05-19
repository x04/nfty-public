use super::shared_types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetEvents {
    #[serde(rename = "asset_events")]
    pub asset_events: Vec<AssetEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetEvent {
    #[serde(rename = "approved_account")]
    pub approved_account: Option<serde_json::Value>,

    #[serde(rename = "asset")]
    pub asset: Option<Asset>,

    #[serde(rename = "asset_bundle")]
    pub asset_bundle: Option<serde_json::Value>,

    #[serde(rename = "auction_type")]
    pub auction_type: Option<String>,

    #[serde(rename = "bid_amount")]
    pub bid_amount: Option<String>,

    #[serde(rename = "collection_slug")]
    pub collection_slug: Option<String>,

    #[serde(rename = "contract_address")]
    pub contract_address: Option<String>,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "custom_event_name")]
    pub custom_event_name: Option<serde_json::Value>,

    #[serde(rename = "dev_fee_payment_event")]
    pub dev_fee_payment_event: Option<serde_json::Value>,

    #[serde(rename = "duration")]
    pub duration: Option<String>,

    #[serde(rename = "ending_price")]
    pub ending_price: String,

    #[serde(rename = "event_type")]
    pub event_type: Option<String>,

    #[serde(rename = "from_account")]
    pub from_account: Option<FromAccount>,

    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "is_private")]
    pub is_private: Option<bool>,

    #[serde(rename = "owner_account")]
    pub owner_account: Option<serde_json::Value>,

    #[serde(rename = "payment_token")]
    pub payment_token: PaymentToken,

    #[serde(rename = "quantity")]
    pub quantity: String,

    #[serde(rename = "seller")]
    pub seller: Option<FromAccount>,

    #[serde(rename = "starting_price")]
    pub starting_price: String,

    #[serde(rename = "to_account")]
    pub to_account: Option<FromAccount>,

    #[serde(rename = "total_price")]
    pub total_price: Option<String>,

    #[serde(rename = "transaction")]
    pub transaction: Option<Transaction>,

    #[serde(rename = "winner_account")]
    pub winner_account: Option<FromAccount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "token_id")]
    pub token_id: String,

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
    pub asset_contract: AssetContract,

    #[serde(rename = "permalink")]
    pub permalink: Option<String>,

    #[serde(rename = "collection")]
    pub collection: Option<Collection>,

    #[serde(rename = "decimals")]
    pub decimals: Option<i64>,

    #[serde(rename = "token_metadata")]
    pub token_metadata: Option<String>,

    #[serde(rename = "owner")]
    pub owner: Option<FromAccount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FromAccount {
    #[serde(rename = "user")]
    pub user: Option<User>,

    #[serde(rename = "profile_img_url")]
    pub profile_img_url: Option<String>,

    #[serde(rename = "address")]
    pub address: Option<String>,

    #[serde(rename = "config")]
    pub config: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentToken {
    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "symbol")]
    pub symbol: String,

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "block_hash")]
    pub block_hash: Option<String>,

    #[serde(rename = "block_number")]
    pub block_number: Option<String>,

    #[serde(rename = "from_account")]
    pub from_account: Option<FromAccount>,

    #[serde(rename = "id")]
    pub id: Option<i64>,

    #[serde(rename = "timestamp")]
    pub timestamp: Option<String>,

    #[serde(rename = "to_account")]
    pub to_account: Option<FromAccount>,

    #[serde(rename = "transaction_hash")]
    pub transaction_hash: Option<String>,

    #[serde(rename = "transaction_index")]
    pub transaction_index: Option<String>,
}
