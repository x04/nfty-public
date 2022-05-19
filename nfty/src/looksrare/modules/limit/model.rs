use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetEventsResponse {
    pub data: Option<Data>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    pub events: Option<Vec<Event>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub hash: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    pub token: Option<Token>,
    pub order: Option<Order>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    pub address: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<i64>,
    #[serde(rename = "floorOrder")]
    pub floor_order: Option<FloorOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FloorOrder {
    pub price: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "isOrderAsk")]
    pub is_order_ask: Option<bool>,
    pub price: Option<String>,
    #[serde(rename = "endTime")]
    pub end_time: Option<String>,
    pub currency: Option<String>,
    pub strategy: Option<String>,
    pub status: Option<Status>,
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
    pub image: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "CANCELLED")]
    Cancelled,
    #[serde(rename = "EXECUTED")]
    Executed,
    #[serde(rename = "VALID")]
    Valid,
}
