use crate::token::Token;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use toml::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub account: Account,
    pub global: Global,
    pub mint: Option<Mint>,
    pub opensea: Option<OpenSea>,
}

impl Config {
    pub fn create_http_client(&self) -> Result<reqwest::Client, crate::Error> {
        match self.global.proxy_url.as_ref() {
            Some(proxy_url) if !proxy_url.is_empty() => Ok(reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .proxy(reqwest::Proxy::all(proxy_url)?)
                .build()?),
            _ => Ok(reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()?),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub private_key: String,
    pub autosolve_api_key: Option<String>,
    pub autosolve_access_token: Option<String>,
    pub transaction_limit: Option<usize>,
    pub dry_run: bool,
    pub simulate: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Mode {
    Mint,
    Drop,
    OpenSeaLimit,
    LooksRareLimit,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Global {
    pub mode: Mode,
    pub proxy_url: Option<String>,
    pub provider_url: String,
    pub flashbots_signer: Option<String>,
    pub relays: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MintMode {
    Flashbots,
    Normal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum IncludeAddressType {
    To,
    From,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateChecks {
    pub address: Option<String>,
    pub function: String,
    pub arguments: Vec<MintArgument>,
    pub return_value: Vec<MintArgument>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PriceFunction {
    pub address: Option<String>,
    pub function: String,
    pub arguments: Vec<MintArgument>,
    pub multiplier: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mint {
    pub mode: MintMode,
    pub contract_address: String,
    pub function: String,
    pub arguments: Vec<MintArgument>,
    pub value: f64,
    pub gas_fee: f64,
    pub priority_fee: Option<f64>,
    pub gas_limit: Option<u64>,
    pub start_time: Option<u64>,
    pub transaction_count: Option<u64>,
    pub include_address_type: Option<IncludeAddressType>,
    pub include_address: Option<String>,
    pub include_method: Option<String>,
    pub state_checks: Option<Vec<StateChecks>>,
    pub price_function: Option<PriceFunction>,
    pub script_identifier: Option<String>,
    pub initial_nonce: Option<u64>,
    pub bump_mempool: Option<bool>,
    pub extra_data: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MintArgument {
    pub r#type: Token,
    pub value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OSAPI {
    Rest,
    GraphQL,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SmartGas {
    Enabled,
    Disabled,
    Exclusive,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpenSea {
    pub api: OSAPI,
    pub api_key: Option<String>,
    pub api_delay: Option<u64>,
    pub smart_gas: SmartGas,
    pub estimate_gas: bool,
    pub gas_fee: f64,
    pub priority_fee: Option<f64>,
    pub gas_limit: u64,
    pub maximum_retry_attempts: usize,
    pub drop: Option<OSDrop>,
    pub limit: Option<OSLimit>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OSDrop {
    pub maximum_price: f64,
    pub max_orders: usize,
    pub token_id: String,
    pub contract_address: String,
    pub listing_username: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OSLimitMode {
    Collection,
    Token,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OSLimit {
    pub mode: OSLimitMode,
    pub collections: Option<Vec<OSLimitCollection>>,
    pub token_id: Option<String>,
    pub contract_address: Option<String>,
    pub minimum_price: Option<f64>,
    pub maximum_price: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum OSLimitTraitType {
    Include,
    Exclude,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OSLimitTrait {
    pub r#type: OSLimitTraitType,
    pub name: String,
    pub value: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OSLimitCollection {
    pub slug: String,
    pub traits: Option<Vec<OSLimitTrait>>,
    pub minimum_price: f64, // in ether, convert to wei with mint_value * 1e18
    pub maximum_price: f64, // in ether, convert to wei with mint_value * 1e18
                            // pub smart_gas: SmartGasType,
}
