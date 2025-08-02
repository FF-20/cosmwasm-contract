#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapStatusResponse {
    pub swap_id: String,
    pub status: SwapStatus,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapResponse {
    pub swap_id: String,
    pub swap_data: Option<SwapData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapListResponse {
    pub swaps: Vec<(String, SwapData)>,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SrcEscrowResponse {
    pub escrow_address: String,
    pub escrow_data: Option<SrcEscrowData>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SrcEscrowListResponse {
    pub escrows: Vec<(String, SrcEscrowData)>,
}
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{Immutables, Order, SrcEscrowData, SwapData, SwapStatus};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub cw20_contract: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CreateDstEscrow {
        escrow_address: String, // User-provided cosmos address
        immutables: Immutables,
        timestamp: Uint128,
    },
    CreateSrcEscrow {
        escrow_address: String,
        order: Order,
        extension: Binary,
        order_hash: String,
        taker: String, // cosmos address as string
        making_amount: Uint128,
        taking_amount: Uint128,
        remaining_making_amount: Uint128,
        extra_data: Binary,
    },
    // Resolver functions
    ExecuteFinalizeSwap {
        swap_id: String,
        eth_tx_hash: String,
    },
    CreateSwap {
        swap_id: String,
        maker: String, // Ethereum maker address as string
        token: String, // Cosmos token address
        amount: Uint128,
    },
    TransferWithPermit {
        permit: Permit,
        from: String,
        to: String,
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ValidateEscrow {
        escrow_address: String,
    },
    GetDstEscrow {
        escrow_address: String, // Changed from escrow_key to escrow_address
    },
    GetSrcEscrow {
        escrow_address: String,
    },
    ListDstEscrows {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    ListSrcEscrows {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    // Resolver queries
    QuerySwapStatus {
        swap_id: String,
    },
    GetSwap {
        swap_id: String,
    },
    ListSwaps {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ValidateEscrowResponse {
    pub is_valid: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DstEscrowResponse {
    pub escrow_address: String, // Changed from escrow_key
    pub immutables: Option<Immutables>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct DstEscrowListResponse {
    pub escrows: Vec<(String, Immutables)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Permit {
    pub params: PermitParams,
    pub signature: PermitSignature,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PermitParams {
    pub permit_name: String,
    pub nonce: u64,
    pub expiration: Option<u64>,
    pub allowed_tokens: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PermitSignature {
    pub signature: Binary,
}