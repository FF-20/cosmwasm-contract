
use crate::error::ContractError;
use crate::state::{Immutables, Order, SrcEscrowData, DST_ESCROWS, SRC_ESCROWS, EVENT_TYPE_DST_ESCROW_CREATED, EVENT_TYPE_SRC_ESCROW_CREATED};

/// Creates a new source escrow
pub fn create_src_escrow(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    escrow_address: String,
    order: Order,
    extension: Binary,
    order_hash: String,
    taker: String,
    making_amount: Uint128,
    taking_amount: Uint128,
    remaining_making_amount: Uint128,
    extra_data: Binary,
) -> Result<Response, ContractError> {
    // Validate the cosmos addresses
    deps.api.addr_validate(&escrow_address)?;
    let taker_addr = deps.api.addr_validate(&taker)?;

    // Check if escrow already exists for this address
    if SRC_ESCROWS.has(deps.storage, escrow_address.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Create the escrow data
    let escrow_data = SrcEscrowData {
        order,
        extension,
        order_hash: order_hash.clone(),
        taker: taker_addr,
        making_amount,
        taking_amount,
        remaining_making_amount,
        extra_data,
    };

    // Store the escrow using the provided address
    SRC_ESCROWS.save(deps.storage, escrow_address.clone(), &escrow_data)?;

    // Create the event
    let event = Event::new(EVENT_TYPE_SRC_ESCROW_CREATED)
        .add_attribute("escrow_address", &escrow_address)
        .add_attribute("order_hash", &order_hash)
        .add_attribute("maker", escrow_data.order.maker.to_string())
        .add_attribute("taker", escrow_data.taker.to_string())
        .add_attribute("maker_asset", escrow_data.order.maker_asset.to_string())
        .add_attribute("taker_asset", escrow_data.order.taker_asset.to_string())
        .add_attribute("making_amount", making_amount.to_string())
        .add_attribute("taking_amount", taking_amount.to_string())
        .add_attribute("remaining_making_amount", remaining_making_amount.to_string())
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_src_escrow")
        .add_attribute("escrow_address", escrow_address))
}use cosmwasm_std::{DepsMut, Env, Event, MessageInfo, Response, Uint128, Binary};


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
    use cosmwasm_std::Binary;

    // Create a deterministic address based on sender and block info
    let input = format!("{}:{}", sender, block.height);
    let hash = cosmwasm_std::to_binary(&input).unwrap();

    // Take first 20 bytes and encode as hex
    format!("escrow{}", hex::encode(&hash.to_vec()[..20.min(hash.len())]))
}