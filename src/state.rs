use cosmwasm_std::{Addr, Uint128, Binary};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Order {
    pub salt: Uint128,
    pub maker: Addr,
    pub receiver: Addr,
    pub maker_asset: Addr,
    pub taker_asset: Addr,
    pub making_amount: Uint128,
    pub taking_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Immutables {
    pub order_hash: String,      // bytes32 as hex string (e.g., "0x...")
    pub hashlock: String,        // same
    pub maker: Addr,
    pub taker: Addr,
    pub token: Addr,
    pub amount: Uint128,
    pub safety_deposit: Uint128,
    pub timelocks: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SrcEscrowData {
    pub order: Order,
    pub extension: Binary,
    pub order_hash: String,      // bytes32 as hex string
    pub taker: Addr,
    pub making_amount: Uint128,
    pub taking_amount: Uint128,
    pub remaining_making_amount: Uint128,
    pub extra_data: Binary,
}

// Storage for destination escrows
pub const DST_ESCROWS: Map<String, Immutables> = Map::new("dst_escrows");

// Storage for source escrows  
pub const SRC_ESCROWS: Map<String, SrcEscrowData> = Map::new("src_escrows");

// Event attribute constants
pub const EVENT_TYPE_DST_ESCROW_CREATED: &str = "dst_escrow_created";
pub const EVENT_TYPE_SRC_ESCROW_CREATED: &str = "src_escrow_created";