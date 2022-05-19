use ethers::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AtomicOrder {
    pub exchange: Address,
    pub maker: Address,
    pub taker: Address,
    pub maker_relayer_fee: U256,
    pub taker_relayer_fee: U256,
    pub maker_protocol_fee: U256,
    pub taker_protocol_fee: U256,
    pub fee_recipient: Address,
    pub fee_method: u8,
    pub side: u8,
    pub sale_kind: u8,
    pub target: Address,
    pub how_to_call: u8,
    pub calldata: Vec<u8>,
    pub replacement_pattern: Vec<u8>,
    pub static_target: Address,
    pub static_extra_data: Vec<u8>,
    pub payment_token: Address,
    pub base_price: U256,
    pub extra: U256,
    pub listing_time: U256,
    pub expiration_time: U256,
    pub salt: U256,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtomicSig {
    pub v: u8,
    pub r: Vec<u8>,
    pub s: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AtomicMatchArgs {
    pub buy: AtomicOrder,
    pub buy_sig: AtomicSig,
    pub sell: AtomicOrder,
    pub sell_sig: AtomicSig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    #[serde(rename = "data")]
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "orders")]
    pub orders: Orders,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Orders {
    #[serde(rename = "edges")]
    pub edges: Vec<Edge>,
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageInfo {
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Edge {
    #[serde(rename = "node")]
    pub node: Node,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "oldOrder")]
    pub old_order: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OldOrder {
    #[serde(rename = "id")]
    pub id: i64,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "closing_extendable")]
    pub closing_extendable: bool,

    #[serde(rename = "expiration_time")]
    pub expiration_time: u64,

    #[serde(rename = "listing_time")]
    pub listing_time: u64,

    #[serde(rename = "order_hash")]
    pub order_hash: Option<String>,

    #[serde(rename = "metadata")]
    pub metadata: Metadata,

    #[serde(rename = "exchange")]
    pub exchange: String,

    #[serde(rename = "maker")]
    pub maker: FeeRecipient,

    #[serde(rename = "taker")]
    pub taker: FeeRecipient,

    #[serde(rename = "current_price")]
    pub current_price: String,

    #[serde(rename = "current_bounty")]
    pub current_bounty: String,

    #[serde(rename = "bounty_multiple")]
    pub bounty_multiple: String,

    #[serde(rename = "maker_relayer_fee")]
    pub maker_relayer_fee: String,

    #[serde(rename = "taker_relayer_fee")]
    pub taker_relayer_fee: String,

    #[serde(rename = "maker_protocol_fee")]
    pub maker_protocol_fee: String,

    #[serde(rename = "taker_protocol_fee")]
    pub taker_protocol_fee: String,

    #[serde(rename = "maker_referrer_fee")]
    pub maker_referrer_fee: String,

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
    pub payment_token_contract: PaymentTokenContract,

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
    pub approved_on_chain: bool,

    #[serde(rename = "cancelled")]
    pub cancelled: bool,

    #[serde(rename = "finalized")]
    pub finalized: bool,

    #[serde(rename = "marked_invalid")]
    pub marked_invalid: bool,

    #[serde(rename = "prefixed_hash")]
    pub prefixed_hash: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetContract {
    #[serde(rename = "address")]
    pub address: Option<String>,

    #[serde(rename = "asset_contract_type")]
    pub asset_contract_type: Option<String>,

    #[serde(rename = "created_date")]
    pub created_date: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "nft_version")]
    pub nft_version: Option<serde_json::Value>,

    #[serde(rename = "owner")]
    pub owner: i64,

    #[serde(rename = "schema_name")]
    pub schema_name: String,

    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "total_supply")]
    pub total_supply: Option<serde_json::Value>,

    #[serde(rename = "description")]
    pub description: Option<String>,

    #[serde(rename = "external_link")]
    pub external_link: Option<String>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "default_to_fiat")]
    pub default_to_fiat: bool,

    #[serde(rename = "dev_buyer_fee_basis_points")]
    pub dev_buyer_fee_basis_points: i64,

    #[serde(rename = "dev_seller_fee_basis_points")]
    pub dev_seller_fee_basis_points: i64,

    #[serde(rename = "only_proxied_transfers")]
    pub only_proxied_transfers: bool,

    #[serde(rename = "opensea_buyer_fee_basis_points")]
    pub opensea_buyer_fee_basis_points: i64,

    #[serde(rename = "opensea_seller_fee_basis_points")]
    pub opensea_seller_fee_basis_points: i64,

    #[serde(rename = "buyer_fee_basis_points")]
    pub buyer_fee_basis_points: i64,

    #[serde(rename = "seller_fee_basis_points")]
    pub seller_fee_basis_points: i64,

    #[serde(rename = "payout_address")]
    pub payout_address: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "username")]
    pub username: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(rename = "schema")]
    pub schema: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaymentTokenContract {
    #[serde(rename = "id")]
    pub id: i64,

    #[serde(rename = "symbol")]
    pub symbol: Option<String>,

    #[serde(rename = "address")]
    pub address: Option<String>,

    #[serde(rename = "image_url")]
    pub image_url: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "decimals")]
    pub decimals: i64,

    #[serde(rename = "eth_price")]
    pub eth_price: Option<String>,

    #[serde(rename = "usd_price")]
    pub usd_price: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSeaAssetsSearch {
    #[serde(rename = "data")]
    pub data: SearchData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchData {
    #[serde(rename = "query")]
    pub query: Query,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    #[serde(rename = "collection")]
    pub collection: Option<SearchCollection>,

    #[serde(rename = "search")]
    pub search: Option<Search>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchCollection {
    #[serde(rename = "numericTraits")]
    pub numeric_traits: Vec<NumericTrait>,

    #[serde(rename = "stringTraits")]
    pub string_traits: Vec<StringTrait>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NumericTrait {
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "value")]
    pub value: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Value {
    #[serde(rename = "max")]
    pub max: f64,

    #[serde(rename = "min")]
    pub min: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StringTrait {
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "counts")]
    pub counts: Vec<Count>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Count {
    #[serde(rename = "count")]
    pub count: i64,

    #[serde(rename = "value")]
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    #[serde(rename = "edges")]
    pub edges: Vec<SearchEdge>,

    #[serde(rename = "totalCount")]
    pub total_count: i64,

    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchEdge {
    #[serde(rename = "node")]
    pub node: SearchNode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchNode {
    #[serde(rename = "asset")]
    pub asset: Option<NodeAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeAsset {
    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "tokenId")]
    pub token_id: String,

    #[serde(rename = "assetContract")]
    pub asset_contract: SearchAssetContract,

    #[serde(rename = "orderData")]
    pub order_data: OrderData,

    #[serde(rename = "traits")]
    pub traits: Traits,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchAssetContract {
    #[serde(rename = "address")]
    pub address: Address,

    #[serde(rename = "chain")]
    pub chain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderData {
    #[serde(rename = "bestAsk")]
    pub best_ask: Option<BestAsk>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BestAsk {
    #[serde(rename = "orderType")]
    pub order_type: String,

    #[serde(rename = "quantity")]
    pub quantity: String,

    #[serde(rename = "decimals")]
    pub decimals: Option<String>,

    #[serde(rename = "paymentAssetQuantity")]
    pub payment_asset_quantity: PaymentAssetQuantity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentAssetQuantity {
    #[serde(rename = "quantity")]
    pub quantity: String,

    #[serde(rename = "asset")]
    pub asset: PaymentAssetQuantityAsset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentAssetQuantityAsset {
    #[serde(rename = "decimals")]
    pub decimals: i64,

    #[serde(rename = "symbol")]
    pub symbol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Traits {
    #[serde(rename = "edges")]
    pub edges: Vec<TraitEdge>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TraitEdge {
    #[serde(rename = "node")]
    pub node: TraitNode,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TraitNode {
    #[serde(rename = "traitType")]
    pub trait_type: String,

    #[serde(rename = "traitCount")]
    pub trait_count: i64,

    #[serde(rename = "value")]
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenSeaEventHistory {
    #[serde(rename = "data")]
    pub data: EventHistoryData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryData {
    #[serde(rename = "assetEvents")]
    pub asset_events: AssetEvents,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetEvents {
    #[serde(rename = "edges")]
    pub edges: Vec<EventHistoryEdge>,

    #[serde(rename = "pageInfo")]
    pub page_info: Option<PageInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryEdge {
    #[serde(rename = "node")]
    pub node: EventHistoryNode,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryNode {
    #[serde(rename = "assetQuantity")]
    pub asset_quantity: Option<AssetQuantity>,

    #[serde(rename = "price")]
    pub price: Option<Price>,

    #[serde(rename = "endingPrice")]
    pub ending_price: Option<Price>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetQuantity {
    #[serde(rename = "asset")]
    pub asset: AssetQuantityAsset,

    #[serde(rename = "quantity")]
    pub quantity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetQuantityAsset {
    #[serde(rename = "name")]
    pub name: Option<String>,
    #[serde(rename = "tokenId")]
    pub token_id: String,
    #[serde(rename = "collection")]
    pub collection: EventHistoryAssetQuantityCollection,
    #[serde(rename = "assetContract")]
    pub contract: EventHistoryAssetQuantityContract,
    #[serde(rename = "traits")]
    pub traits: Option<Traits>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryAssetQuantityCollection {
    #[serde(rename = "slug")]
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryAssetQuantityContract {
    #[serde(rename = "address")]
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    #[serde(rename = "asset")]
    pub asset: EndingPriceAsset,

    #[serde(rename = "quantity")]
    pub quantity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EndingPriceAsset {
    #[serde(rename = "symbol")]
    pub symbol: String,

    #[serde(rename = "assetContract")]
    pub asset_contract: EventHistoryAssetContract,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventHistoryAssetContract {
    #[serde(rename = "chain")]
    pub chain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EtherscanGas {
    #[serde(rename = "status")]
    pub status: String,

    #[serde(rename = "message")]
    pub message: String,

    #[serde(rename = "result")]
    pub result: EtherscanGasResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EtherscanGasResult {
    #[serde(rename = "LastBlock")]
    pub last_block: String,

    #[serde(rename = "SafeGasPrice")]
    pub safe_gas_price: String,

    #[serde(rename = "ProposeGasPrice")]
    pub propose_gas_price: String,

    #[serde(rename = "FastGasPrice")]
    pub fast_gas_price: String,

    #[serde(rename = "suggestBaseFee")]
    pub suggest_base_fee: String,

    #[serde(rename = "gasUsedRatio")]
    pub gas_used_ratio: String,
}
