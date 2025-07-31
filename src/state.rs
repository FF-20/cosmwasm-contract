use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Immutables {
    pub order_hash: String,      // bytes32 as hex string (e.g., "0x...")
    pub hashlock: String,        // String for now. Change later.   
    pub maker: Addr,
    pub taker: Addr,
    pub token: Addr,
    pub amount: Uint128,
    pub safety_deposit: Uint128,
    pub timelocks: Uint128,
}

// Storage for destination escrows
pub const DST_ESCROWS: Map<String, Immutables> = Map::new("dst_escrows");

// Event attribute constants
pub const EVENT_TYPE_DST_ESCROW_CREATED: &str = "dst_escrow_created";