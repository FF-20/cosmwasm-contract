use cosmwasm_std::{Addr, Binary, Uint128};
use cw_storage_plus::Map;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Order { // The order info used to create SRC. Filled by frontend.
    pub salt: String, // Is this needed?
    pub maker: String, // Maker (cos) address.
    pub taker: Option<String>, // Resolver (eth) address.
    pub secret_hash: String, // Hashlock
    pub maker_asset: Option<String>, // COS
    pub taker_asset: Option<String>, // ETH
    pub making_amount: Option<Uint128>,
    pub taking_amount: Option<Uint128>,
    pub signature: String, // Order signature.
    pub timelock: TimeLocks,
    pub nonce: Option<String>,
    pub created_at: Option<Uint128>,
    pub status: Option<Uint128>, // TODO: Add OrderStatus
    pub est_gas_cost: Option<Uint128>, // TODO: Add OrderStatus
    pub profitability: Option<String>, // TODO: Add OrderStatus

}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Immutables { // The order infor used to create the DST.
    pub order_hash: String, // bytes32 as hex string (e.g., "0x...") // NOT REDUNDANT.
    pub secret_hash: String,   // Hashlock
    pub maker: Addr,
    pub taker: Addr,
    pub token: Addr,
    pub amount: Uint128,
    pub safety_deposit: Uint128,
    pub timelocks: TimeLocks,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SrcEscrowData {
    pub order: Order,
    pub extension: Option<Binary>,
    pub order_hash: Option<String>, // bytes32 as hex string NOT HASHLOCK
    pub taker: Option<Addr>, // Also the resolver stated in the Order?
    pub remaining_making_amount: Uint128,
    pub src_chain_id: String, // COS id
    pub dst_chain_id: String, // ETH id
    pub extra_data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SwapStatus {
    Pending,
    Completed,
    Unknown,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapData {
    pub swap_id: String,
    pub maker: Addr,     // Ethereum maker address (as string for cross-chain)
    pub taker: Addr,     // Cosmos taker address
    pub token: Addr,     // Cosmos token to release
    pub amount: Uint128, // Amount to release to maker
    pub eth_tx_hash: Option<String>, // Ethereum transaction hash (optional)
    pub status: SwapStatus,
    pub created_at: u64, // Block timestamp
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeLocks {
    pub src_withdrawal: u64,
    pub src_public_withdrawal: u64,
    pub src_cancellation: u64,
    pub src_public_cancellation: u64,
    pub dst_withdrawal: u64,
    pub dst_public_withdrawal: u64,
    pub dst_cancellation: u64,
}

// Storage for destination escrows
pub const DST_ESCROWS: Map<String, Immutables> = Map::new("dst_escrows");

// Storage for source escrows
pub const SRC_ESCROWS: Map<String, SrcEscrowData> = Map::new("src_escrows");

// Storage for atomic swaps
pub const SWAPS: Map<String, SwapData> = Map::new("swaps");

// Event attribute constants
pub const EVENT_TYPE_DST_ESCROW_CREATED: &str = "dst_escrow_created";
pub const EVENT_TYPE_SRC_ESCROW_CREATED: &str = "src_escrow_created";
pub const EVENT_TYPE_SWAP_FINALIZED: &str = "swap_finalized";
pub const EVENT_TYPE_SWAP_CREATED: &str = "swap_created";
