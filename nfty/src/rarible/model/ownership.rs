#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub type Ownerships = Vec<Owner>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "tokenId")]
    pub token_id: String,

    #[serde(rename = "owner")]
    pub owner: String,

    #[serde(rename = "value")]
    pub value: u128,

    #[serde(rename = "date")]
    pub date: String,

    #[serde(rename = "status")]
    pub status: Status,

    #[serde(rename = "selling")]
    pub selling: i64,

    #[serde(rename = "sold")]
    pub sold: i64,

    #[serde(rename = "stock")]
    pub stock: i64,

    #[serde(rename = "pending")]
    pub pending: Vec<Option<serde_json::Value>>,

    #[serde(rename = "blacklisted")]
    pub blacklisted: bool,

    #[serde(rename = "creator")]
    pub creator: Option<String>,

    #[serde(rename = "verified")]
    pub verified: bool,

    #[serde(rename = "categories")]
    pub categories: Vec<Option<serde_json::Value>>,

    #[serde(rename = "likes")]
    pub likes: Option<i64>,

    #[serde(rename = "hide")]
    pub hide: Option<bool>,

    #[serde(rename = "lazyValue")]
    pub lazy_value: Option<i64>,

    #[serde(rename = "version")]
    pub version: Option<i64>,

    #[serde(rename = "price")]
    pub price: Option<f64>,

    #[serde(rename = "priceEth")]
    pub price_eth: Option<f64>,

    #[serde(rename = "buyToken")]
    pub buy_token: Option<String>,

    #[serde(rename = "buyTokenId")]
    pub buy_token_id: Option<String>,

    #[serde(rename = "signature")]
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[serde(rename = "FIXED_PRICE")]
    FixedPrice,

    #[serde(rename = "NOT_FOR_SALE")]
    NotForSale,

    #[serde(rename = "OPEN_FOR_OFFERS")]
    OpenForOffers,
}
