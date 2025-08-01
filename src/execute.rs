/// Creates a new swap for atomic cross-chain exchange
pub fn create_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_id: String,
    maker: String,
    token: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // Validate addresses
    let token_addr = deps.api.addr_validate(&token)?;

    // Check if swap already exists
    if SWAPS.has(deps.storage, swap_id.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Create swap data
    let swap_data = SwapData {
        swap_id: swap_id.clone(),
        maker: deps.api.addr_validate(&maker)?, // Store as Addr for consistency
        taker: info.sender.clone(),
        token: token_addr,
        amount,
        eth_tx_hash: None,
        status: SwapStatus::Pending,
        created_at: env.block.time.seconds(),
    };

    // Store the swap
    SWAPS.save(deps.storage, swap_id.clone(), &swap_data)?;

    // Create event
    let event = Event::new(EVENT_TYPE_SWAP_CREATED)
        .add_attribute("swap_id", &swap_id)
        .add_attribute("maker", maker)
        .add_attribute("taker", info.sender.to_string())
        .add_attribute("token", swap_data.token.to_string())
        .add_attribute("amount", amount.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_swap")
        .add_attribute("swap_id", swap_id))
}

/// Main entrypoint to finalize a cross-chain atomic swap
pub fn execute_finalize_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_id: String,
    eth_tx_hash: String,
) -> Result<Response, ContractError> {
    // Load swap data
    let mut swap_data = SWAPS.load(deps.storage, swap_id.clone())?;

    // Check if already completed
    if matches!(swap_data.status, SwapStatus::Completed) {
        return Err(ContractError::EscrowAlreadyExists {}); // Reusing error for "already completed"
    }

    // Validate Ethereum transaction (simplified - in production you'd verify the tx)
    validate_eth_fill(&eth_tx_hash)?;

    // Mark swap as completed
    mark_swap_completed(deps.storage, &mut swap_data, eth_tx_hash.clone())?;

    // Release tokens to maker
    let release_msg = release_to_maker(&swap_data)?;

    // Create finalization event
    let event = Event::new(EVENT_TYPE_SWAP_FINALIZED)
        .add_attribute("swap_id", &swap_id)
        .add_attribute("eth_tx_hash", &eth_tx_hash)
        .add_attribute("maker", swap_data.maker.to_string())
        .add_attribute("amount", swap_data.amount.to_string())
        .add_attribute("token", swap_data.token.to_string())
        .add_attribute("finalizer", info.sender.to_string());

    Ok(Response::new()
        .add_message(release_msg)
        .add_event(event)
        .add_attribute("action", "finalize_swap")
        .add_attribute("swap_id", swap_id))
}

/// Validates that the Ethereum transaction hash corresponds to a successful fill
fn validate_eth_fill(eth_tx_hash: &str) -> Result<(), ContractError> {
    // Basic validation - in production this would:
    // 1. Query Ethereum node/indexer to verify transaction exists
    // 2. Parse transaction logs to confirm maker sent funds to resolver
    // 3. Verify transaction is confirmed with sufficient blocks

    if eth_tx_hash.len() != 66 || !eth_tx_hash.starts_with("0x") {
        return Err(ContractError::InvalidTimestamp {}); // Reusing error for invalid tx hash
    }

    Ok(())
}

/// Sends the taker's tokens from contract balance to the maker
fn release_to_maker(swap_data: &SwapData) -> Result<BankMsg, ContractError> {
    // Create bank message to send tokens to maker
    let coin = Coin {
        denom: swap_data.token.to_string(), // Assuming token address represents denom
        amount: swap_data.amount,
    };

    Ok(BankMsg::Send {
        to_address: swap_data.maker.to_string(),
        amount: vec![coin],
    })
}

/// @notice Marks the swap as completed and updates storage
/// @param storage - The mutable storage of the contract, see cosmwasm_std::Storage
/// @param swap_data - The data of the swap, see crate::state::SwapData
/// @param eth_tx_hash - The Ethereum transaction hash.
/// @return A result indicating success or an error, see crate::error::ContractError
fn mark_swap_completed(
    storage: &mut dyn cosmwasm_std::Storage,
    swap_data: &mut SwapData,
    eth_tx_hash: String,
) -> Result<(), ContractError> {
    swap_data.status = SwapStatus::Completed;
    swap_data.eth_tx_hash = Some(eth_tx_hash);

    SWAPS.save(storage, swap_data.swap_id.clone(), swap_data)?;
    Ok(())
}
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
        .add_attribute(
            "remaining_making_amount",
            remaining_making_amount.to_string(),
        )
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_src_escrow")
        .add_attribute("escrow_address", escrow_address))
}
use cosmwasm_std::{BankMsg, Binary, Coin, DepsMut, Env, Event, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::state::{
    Immutables, Order, SrcEscrowData, SwapData, SwapStatus, DST_ESCROWS,
    EVENT_TYPE_DST_ESCROW_CREATED, EVENT_TYPE_SRC_ESCROW_CREATED, EVENT_TYPE_SWAP_CREATED,
    EVENT_TYPE_SWAP_FINALIZED, SRC_ESCROWS, SWAPS,
};

/// Creates a new destination escrow
pub fn create_dst_escrow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    escrow_address: String, // User-provided cosmos address
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
        .add_attribute("escrow_address", &escrow_address) // Changed from escrow_key
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
        .add_attribute("escrow_address", escrow_address)) // Changed from escrow_key
}

// Alternative function to generate a more cosmos-like address
#[allow(dead_code)]
fn generate_escrow_address(sender: &cosmwasm_std::Addr, block: &cosmwasm_std::BlockInfo) -> String {
    let input = format!("{}:{}", sender, block.height);
    let hash = cosmwasm_std::to_binary(&input).unwrap(); // 32-byte SHA-256
    let addr_part = to_hex(&hash[..20]); // first 20 bytes → hex
    format!("escrow{}", addr_part)
}

fn to_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}
