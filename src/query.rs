use cosmwasm_std::{Deps, DepsMut, Env, Event, MessageInfo, Order, Response, StdResult, Uint128};
use crate::error::ContractError;
use crate::state::{Immutables, DST_ESCROWS, EVENT_TYPE_DST_ESCROW_CREATED};
use crate::msg::{DstEscrowListResponse, DstEscrowResponse};

/// Creates a new destination escrow
pub fn create_dst_escrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    escrow_address: String,  // User-provided cosmos address
    immutables: Immutables,
    timestamp: Uint128,
) -> Result<Response, ContractError> {
    // Validate timestamp (ensure it's not in the past)
    if timestamp < Uint128::from(env.block.time.seconds()) {
        return Err(ContractError::InvalidTimestamp {});
    }

    // Validate the cosmos address format
    deps.api.addr_validate(&escrow_address)?;

    // Check if escrow already exists for this address
    if DST_ESCROWS.has(deps.storage, escrow_address.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Store the escrow using the provided address
    DST_ESCROWS.save(deps.storage, escrow_address.clone(), &immutables)?;

    // Create the event
    let event = Event::new(EVENT_TYPE_DST_ESCROW_CREATED)
        .add_attribute("escrow_address", &escrow_address)  // Changed from escrow_key
        .add_attribute("order_hash", &immutables.order_hash)
        .add_attribute("hashlock", &immutables.hashlock)
        .add_attribute("maker", immutables.maker.to_string())
        .add_attribute("taker", immutables.taker.to_string())
        .add_attribute("token", immutables.token.to_string())
        .add_attribute("amount", immutables.amount.to_string())
        .add_attribute("safety_deposit", immutables.safety_deposit.to_string())
        .add_attribute("timelocks", immutables.timelocks.to_string())
        .add_attribute("timestamp", timestamp.to_string())
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_dst_escrow")
        .add_attribute("escrow_address", escrow_address))  // Changed from escrow_key
}

// Alternative function to generate a more cosmos-like address
#[allow(dead_code)]
fn generate_escrow_address(sender: &cosmwasm_std::Addr, block: &cosmwasm_std::BlockInfo) -> String {
    // Create a deterministic address based on sender and block info
    let input = format!("{}:{}", sender, block.height);
    let hash = cosmwasm_std::to_json_binary(&input).unwrap();

    // Take first 20 bytes and encode as hex
    format!("escrow{}", hex::encode(&hash.to_vec()[..20.min(hash.len())]))
}


/// Query a specific destination escrow
pub fn query_dst_escrow(deps: Deps, escrow_address: String) -> StdResult<DstEscrowResponse> {
    let immutables = DST_ESCROWS.may_load(deps.storage, escrow_address.clone())?;

    Ok(DstEscrowResponse {
        escrow_address,  // Changed from escrow_key
        immutables,
    })
}

/// Query all destination escrows with pagination
pub fn query_all_dst_escrows(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<DstEscrowListResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;

    let start = start_after
        .as_deref()
        .map(|s| cw_storage_plus::Bound::exclusive(s));

    let escrows: StdResult<Vec<(String, Immutables)>> = DST_ESCROWS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    Ok(DstEscrowListResponse {
        escrows: escrows?,
    })
}