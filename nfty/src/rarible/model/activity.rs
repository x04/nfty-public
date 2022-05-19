use serde::{Deserialize, Serialize};

pub type Activity = Vec<ActivityElement>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityElement {
    #[serde(rename = "@type")]
    pub activity_type: ActivityType,

    #[serde(rename = "date")]
    pub date: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "owner")]
    pub owner: String,

    #[serde(rename = "buyToken")]
    pub buy_token: Option<String>,

    #[serde(rename = "buyTokenId")]
    pub buy_token_id: Option<String>,

    #[serde(rename = "buyValue")]
    pub buy_value: Option<f64>,

    #[serde(rename = "hash")]
    pub hash: Option<String>,

    #[serde(rename = "price")]
    pub price: Option<f64>,

    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "tokenId")]
    pub token_id: String,

    #[serde(rename = "value")]
    pub value: i64,

    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ActivityType {
    #[serde(rename = "mint")]
    Mint,

    #[serde(rename = "order")]
    Order,

    #[serde(rename = "cancel")]
    Cancel,
}
