use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error as StdError};

pub type Error = Box<dyn StdError + Send + Sync>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaTokenRequest {
    pub task_id: String,
    pub api_key: String,
    pub created_at: u64,
    pub url: String,
    pub site_key: String,
    pub version: i64,
    pub action: String,
    pub min_score: f32,
    pub proxy: String,
    pub proxy_required: bool,
    pub user_agent: String,
    pub cookies: String,
    pub render_parameters: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaTokenResponse {
    pub task_id: String,
    pub api_key: String,
    pub created_at: i64,
    pub request: CaptchaTokenRequest,
    pub token: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: i64,
    pub raw_id: String,
    pub access_token: String,
    pub raw_access_token: String,
    pub api_key: String,
    pub raw_api_key: String,
}
