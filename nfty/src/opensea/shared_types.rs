use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetContract {
    #[serde(rename = "address")]
    pub address: String,

    #[serde(rename = "asset_contract_type")]
    pub asset_contract_type: Option<String>,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "nft_version")]
    pub nft_version: Option<serde_json::Value>,

    #[serde(rename = "opensea_version")]
    pub opensea_version: Option<serde_json::Value>,

    #[serde(rename = "owner")]
    pub owner: Option<serde_json::Value>,

    #[serde(rename = "schema_name")]
    pub schema_name: Option<String>,

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "total_supply")]
    pub total_supply: Option<serde_json::Value>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "external_link")]
    pub external_link: Option<String>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "default_to_fiat")]
    pub default_to_fiat: Option<bool>,

    #[serde(rename = "dev_buyer_fee_basis_points")]
    pub dev_buyer_fee_basis_points: Option<i64>,

    #[serde(rename = "dev_seller_fee_basis_points")]
    pub dev_seller_fee_basis_points: Option<i64>,

    #[serde(rename = "only_proxied_transfers")]
    pub only_proxied_transfers: Option<bool>,

    #[serde(rename = "opensea_buyer_fee_basis_points")]
    pub opensea_buyer_fee_basis_points: Option<i64>,

    #[serde(rename = "opensea_seller_fee_basis_points")]
    pub opensea_seller_fee_basis_points: Option<i64>,

    #[serde(rename = "buyer_fee_basis_points")]
    pub buyer_fee_basis_points: Option<i64>,

    #[serde(rename = "seller_fee_basis_points")]
    pub seller_fee_basis_points: Option<i64>,

    #[serde(rename = "payout_address")]
    pub payout_address: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "banner_image_url")]
    pub banner_image_url: Option<String>,

    #[serde(rename = "chat_url")]
    pub chat_url: Option<serde_json::Value>,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "default_to_fiat")]
    pub default_to_fiat: Option<bool>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "dev_buyer_fee_basis_points")]
    pub dev_buyer_fee_basis_points: Option<String>,

    #[serde(rename = "dev_seller_fee_basis_points")]
    pub dev_seller_fee_basis_points: Option<String>,

    #[serde(rename = "discord_url")]
    pub discord_url: Option<String>,

    #[serde(rename = "display_data")]
    pub display_data: Option<DisplayData>,

    #[serde(rename = "external_url")]
    pub external_url: Option<String>,

    #[serde(rename = "featured")]
    pub featured: Option<bool>,

    #[serde(rename = "featured_image_url")]
    pub featured_image_url: Option<String>,

    #[serde(rename = "hidden")]
    pub hidden: Option<bool>,

    #[serde(rename = "safelist_request_status")]
    pub safelist_request_status: Option<String>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "is_subject_to_whitelist")]
    pub is_subject_to_whitelist: Option<bool>,

    #[serde(rename = "large_image_url")]
    pub large_image_url: Option<String>,

    #[serde(rename = "medium_username")]
    pub medium_username: Option<serde_json::Value>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "only_proxied_transfers")]
    pub only_proxied_transfers: Option<bool>,

    #[serde(rename = "opensea_buyer_fee_basis_points")]
    pub opensea_buyer_fee_basis_points: Option<String>,

    #[serde(rename = "opensea_seller_fee_basis_points")]
    pub opensea_seller_fee_basis_points: Option<String>,

    #[serde(rename = "payout_address")]
    pub payout_address: Option<String>,

    #[serde(rename = "require_email")]
    pub require_email: Option<bool>,

    #[serde(rename = "short_description")]
    pub short_description: Option<serde_json::Value>,

    #[serde(rename = "slug")]
    pub slug: Option<String>,

    #[serde(rename = "telegram_url")]
    pub telegram_url: Option<serde_json::Value>,

    #[serde(rename = "twitter_username")]
    pub twitter_username: Option<String>,

    #[serde(rename = "instagram_username")]
    pub instagram_username: Option<String>,

    #[serde(rename = "wiki_url")]
    pub wiki_url: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayData {
    #[serde(rename = "card_display_style")]
    pub card_display_style: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "username")]
    pub username: Option<String>,
}
