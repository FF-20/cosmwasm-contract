use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::state::{Immutables, DST_ESCROWS, EVENT_TYPE_DST_ESCROW_CREATED};

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