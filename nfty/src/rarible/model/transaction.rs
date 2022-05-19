use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PreparedTransaction {
    #[serde(rename = "asset")]
    pub asset: Asset,

    #[serde(rename = "transaction")]
    pub transaction: Transaction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    #[serde(rename = "assetType")]
    pub asset_type: AssetType,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetType {
    #[serde(rename = "assetClass")]
    pub asset_class: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "data")]
    pub data: String,

    #[serde(rename = "to")]
    pub to: String,
}
