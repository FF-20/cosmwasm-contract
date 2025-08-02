// use cosmwasm_std::{
//     to_binary, Binary, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response, Uint128, WasmMsg,
// };
use crate::state::{Config, CONFIG, NONCES, USED_PERMITS};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, Permit, QueryMsg};

use cosmwasm_std::{attr};

use crate::permit::verify_permit_signature;

use cw20::Cw20ExecuteMsg;

/// Creates a new swap for atomic cross-chain exchange
/// This function is ass. Needs to be changed.
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
    // let token_addr = &token;

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

/// Transfer maker's asset.
/// Main entrypoint to finalize a cross-chain atomic swap
/// perhaps we don't need this.
pub fn execute_finalize_swap(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    swap_id: String, // We cannot do swap_id. We have to do get the SWAP based on
    eth_tx_hash: String,
) -> Result<Response, ContractError> {
    // Load swap data
    let mut swap_data = SWAPS.load(deps.storage, swap_id.clone())?;

    // Check if already completed
    if matches!(swap_data.status, SwapStatus::Completed) {
        return Err(ContractError::EscrowAlreadyExists {}); // Reusing error for "already completed"
    }

    // Validate Ethereum transaction (simplified - in production you'd verify the tx)
    validate_eth_fill(&eth_tx_hash)?; // We don't need this.

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

/// Creates a new regular source escrow
pub fn create_src_escrow(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    escrow_address: String,
    order: Order,      // Order info.
    extension: Binary, // ?
    // order_hash: String, //
    taker: String,                    // Resolver address.
    making_amount: Uint128,           // Maker amount.
    taking_amount: Uint128,           // Taker amount.
    remaining_making_amount: Uint128, // ?
    extra_data: Binary,               // Additional data.
) -> Result<Response, ContractError> {
    // Validate the cosmos addresses
    // deps.api.addr_validate(&escrow_address)?; // No need for cosmos src escrow to have valid address.
    let taker_addr = deps.api.addr_validate(&taker)?;
    let order_hash = "aa".to_string(); // TODO: Implement get_order_hash() fn

    // Check if escrow already exists for this address
    if SRC_ESCROWS.has(deps.storage, escrow_address.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Create the escrow data
    let escrow_data = SrcEscrowData {
        order,
        // extension,
        order_hash: Some(order_hash.clone()),
        taker: Some(taker_addr),
        extension: Some(extension),
        src_chain_id: "pion-1".to_string(),   // Neutron
        dst_chain_id: "11155111".to_string(), // Sepolia
        // making_amount,
        // taking_amount,
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
        // .add_attribute("taker", escrow_data.taker.to_string())
        .add_attribute(
            "taker",
            escrow_data
                .taker
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "maker_asset",
            escrow_data
                .order
                .maker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "taker_asset",
            escrow_data
                .order
                .taker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        // .add_attribute("maker_asset", escrow_data.order.maker_asset.to_string())
        // .add_attribute("taker_asset", escrow_data.order.taker_asset.to_string())
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

/// Create a new source escrow that we can transfer to.
pub fn create_src_escrow_transfer(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    // escrow_address: String, // No need for this.
    mut order: Order,  // Make it mutable to update making_amount
    extension: Binary, // ?
    // order_hash: String, //
    taker: String,                    // Resolver address.
    making_amount: Uint128,           // Maker amount.
    taking_amount: Uint128,           // Taker amount.
    remaining_making_amount: Uint128, // ?
    extra_data: Binary,               // Additional data.
) -> Result<Response, ContractError> {
    // Validate the cosmos addresses
    let taker_addr = deps.api.addr_validate(&taker)?;
    let order_hash = "aa".to_string(); // TODO: Implement get_order_hash() fn

    // Check if escrow already exists for this address
    // if SRC_ESCROWS.has(deps.storage, escrow_address.clone()) {
    //     return Err(ContractError::EscrowAlreadyExists {});
    // }
    let hashlock = &order.secret_hash.clone();
    if SRC_ESCROWS.has(deps.storage, hashlock.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Validate and record the funds sent with the transaction
    if info.funds.is_empty() {
        return Err(ContractError::InvalidTimestamp {}); // Reusing error - no funds sent
    }

    // Find the expected token in the sent funds
    let expected_denom = order
        .maker_asset
        .as_ref()
        .ok_or(ContractError::InvalidTimestamp {})? // Reusing error - no maker_asset specified
        .clone();

    let sent_funds = info
        .funds
        .iter()
        .find(|coin| coin.denom == expected_denom)
        .ok_or(ContractError::InvalidTimestamp {})?; // Reusing error - wrong token sent

    // Verify the sent amount matches the expected making_amount
    if sent_funds.amount != making_amount {
        return Err(ContractError::InvalidTimestamp {}); // Reusing error - wrong amount
    }

    // Update the order with the actual received amount
    order.making_amount = Some(making_amount);

    // Create the escrow data
    let escrow_data = SrcEscrowData {
        order,
        order_hash: Some(order_hash.clone()),
        taker: Some(taker_addr),
        extension: Some(extension),
        src_chain_id: "pion-1".to_string(),   // Neutron
        dst_chain_id: "11155111".to_string(), // Sepolia
        remaining_making_amount,
        extra_data,
    };

    // Store the escrow using the provided address
    // SRC_ESCROWS.save(deps.storage, escrow_address.clone(), &escrow_data)?;
    SRC_ESCROWS.save(deps.storage, hashlock.clone(), &escrow_data)?;

    // Create the event
    let event = Event::new(EVENT_TYPE_SRC_ESCROW_CREATED)
        .add_attribute("hashlock", hashlock)
        .add_attribute("order_hash", &order_hash)
        .add_attribute("maker", escrow_data.order.maker.to_string())
        .add_attribute(
            "taker",
            escrow_data
                .taker
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "maker_asset",
            escrow_data
                .order
                .maker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "taker_asset",
            escrow_data
                .order
                .taker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute("making_amount", making_amount.to_string())
        .add_attribute("taking_amount", taking_amount.to_string())
        .add_attribute(
            "remaining_making_amount",
            remaining_making_amount.to_string(),
        )
        .add_attribute("funds_received", sent_funds.amount.to_string())
        .add_attribute("funds_denom", &sent_funds.denom)
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_src_escrow")
        .add_attribute("hashlock", hashlock))
}

// Helper function for other functions to release funds from escrow
pub fn release_escrow_funds(
    deps: DepsMut,
    escrow_address: String,
    recipient: String,
) -> Result<Response, ContractError> {
    // Load the escrow data
    let escrow_data = SRC_ESCROWS.load(deps.storage, escrow_address.clone())?;

    // Validate recipient address
    let recipient_addr = deps.api.addr_validate(&recipient)?;

    // Create the bank message to send funds
    let amount = escrow_data.order.making_amount.unwrap_or_default();
    let denom = escrow_data
        .order
        .maker_asset
        .as_ref()
        .ok_or(ContractError::InvalidTimestamp {})?; // Reusing error

    let send_msg = cosmwasm_std::BankMsg::Send {
        to_address: recipient_addr.to_string(),
        amount: vec![Coin {
            denom: denom.clone(),
            amount,
        }],
    };

    // Remove the escrow since funds are released
    SRC_ESCROWS.remove(deps.storage, escrow_address.clone());

    let event = Event::new("escrow_funds_released")
        .add_attribute("escrow_address", &escrow_address)
        .add_attribute("recipient", recipient_addr.to_string())
        .add_attribute("amount", amount.to_string())
        .add_attribute("denom", denom);

    Ok(Response::new()
        .add_message(send_msg)
        .add_event(event)
        .add_attribute("action", "release_escrow_funds")
        .add_attribute("escrow_address", escrow_address))
}

/// Creates a new source escrow from a maker signature
pub fn create_src_escrow_sign(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    escrow_address: String,
    order: Order,      // Order info.
    extension: Binary, // ?
    // order_hash: String, //
    taker: String,                    // Resolver address.
    making_amount: Uint128,           // Maker amount.
    taking_amount: Uint128,           // Taker amount.
    remaining_making_amount: Uint128, // ?
    extra_data: Binary,               // Additional data.
) -> Result<Response, ContractError> {
    // Validate the cosmos addresses
    // deps.api.addr_validate(&escrow_address)?; // No need for cosmos src escrow to have valid address.
    let taker_addr = deps.api.addr_validate(&taker)?;
    let order_hash = "aa".to_string(); // TODO: Implement get_order_hash() fn

    // Check if escrow already exists for this address
    if SRC_ESCROWS.has(deps.storage, escrow_address.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Create the escrow data
    let escrow_data = SrcEscrowData {
        order,
        // extension,
        order_hash: Some(order_hash.clone()),
        taker: Some(taker_addr),
        extension: Some(extension),
        src_chain_id: "pion-1".to_string(),   // Neutron
        dst_chain_id: "11155111".to_string(), // Sepolia
        // making_amount,
        // taking_amount,
        remaining_making_amount,
        extra_data,
    };

    // Verify signature and create CW20 transfer message
    let transfer_msg = verify_signature_and_create_cw20_transfer(
        &deps,
        &escrow_data.order.signature,
        &escrow_data.order.maker,
        &escrow_data
            .order
            .maker_asset
            .as_ref()
            .ok_or(ContractError::InvalidTimestamp {})?, // Reusing error
        making_amount,
        &order_hash,
        &env,
    )?;

    // Store the escrow using the provided address
    SRC_ESCROWS.save(deps.storage, escrow_address.clone(), &escrow_data)?;

    // Create the event
    let event = Event::new(EVENT_TYPE_SRC_ESCROW_CREATED)
        .add_attribute("escrow_address", &escrow_address)
        .add_attribute("order_hash", &order_hash)
        .add_attribute("maker", escrow_data.order.maker.to_string())
        // .add_attribute("taker", escrow_data.taker.to_string())
        .add_attribute(
            "taker",
            escrow_data
                .taker
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "maker_asset",
            escrow_data
                .order
                .maker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        .add_attribute(
            "taker_asset",
            escrow_data
                .order
                .taker_asset
                .as_ref()
                .map_or("none".to_string(), |addr| addr.to_string()),
        )
        // .add_attribute("maker_asset", escrow_data.order.maker_asset.to_string())
        // .add_attribute("taker_asset", escrow_data.order.taker_asset.to_string())
        .add_attribute("making_amount", making_amount.to_string())
        .add_attribute("taking_amount", taking_amount.to_string())
        .add_attribute(
            "remaining_making_amount",
            remaining_making_amount.to_string(),
        )
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_message(transfer_msg) // Add the CW20 transfer message
        .add_event(event)
        .add_attribute("action", "create_src_escrow")
        .add_attribute("escrow_address", escrow_address))
}

// Used by create_src_escrow_sign.
fn verify_signature_and_create_cw20_transfer(
    deps: &DepsMut,
    signature: &str,
    maker: &str,
    maker_asset_contract: &str, // CW20 contract address
    amount: Uint128,
    order_hash: &str,
    env: &Env,
) -> Result<CosmosMsg, ContractError> {
    // Step 1: Verify the signature
    // NOTTODO: Implement signature verification logic here
    // This would typically involve:
    // 1. Reconstruct the message that was signed (order_hash + amount + maker_asset + nonce)
    // 2. Recover the public key from the signature
    // 3. Verify that the recovered address matches the maker
    // 4. Check nonce to prevent replay attacks

    // For now, we'll assume signature is valid and proceed
    // In production, you'd want something like:
    // let message = format!("{}{}{}{}", order_hash, amount, maker_asset_contract, nonce);
    // let is_valid = verify_secp256k1_signature(signature, &message, maker)?;
    // if !is_valid { return Err(ContractError::InvalidSignature {}); }

    // Step 2: Create CW20 TransferFrom message
    // This assumes the maker has already approved this contract to spend their tokens
    // via a separate approve transaction, OR we implement a permit system
    let transfer_from_msg = Cw20ExecuteMsg::TransferFrom {
        owner: maker.to_string(),
        recipient: env.contract.address.to_string(), // Transfer to this contract
        amount,
    };

    let wasm_msg = WasmMsg::Execute {
        contract_addr: maker_asset_contract.to_string(),
        msg: to_binary(&transfer_from_msg)?,
        funds: vec![],
    };

    Ok(CosmosMsg::Wasm(wasm_msg))
}

use cosmwasm_std::{
    to_binary, BankMsg, Binary, Coin, CosmosMsg, DepsMut, Env, Event, MessageInfo, Response,
    Uint128, WasmMsg,
};

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
    escrow_address: String, // Resolver-provided neutron address
    // hashlock: String,
    immutables: Immutables,
    timestamp: Uint128,
) -> Result<Response, ContractError> {
    // Validate timestamp (ensure it's not in the past)
    if timestamp < Uint128::from(env.block.time.seconds()) {
        return Err(ContractError::InvalidTimestamp {});
    }

    // Check if escrow already exists for this address
    if DST_ESCROWS.has(deps.storage, immutables.secret_hash.clone()) {
        return Err(ContractError::EscrowAlreadyExists {});
    }

    // Store the escrow using the provided address
    DST_ESCROWS.save(deps.storage, immutables.secret_hash.clone(), &immutables)?;

    // Create the event
    let event = Event::new(EVENT_TYPE_DST_ESCROW_CREATED)
        .add_attribute("escrow_address", &escrow_address) // Changed from escrow_key
        .add_attribute("order_hash", &immutables.order_hash)
        .add_attribute("hashlock", &immutables.secret_hash) 
        .add_attribute("maker", immutables.maker.to_string())
        .add_attribute("taker", immutables.taker.to_string())
        .add_attribute("token", immutables.token.to_string())
        .add_attribute("amount", immutables.amount.to_string())
        .add_attribute("safety_deposit", immutables.safety_deposit.to_string())
        .add_attribute("timelocks", serde_json::to_string(&immutables.timelocks).unwrap_or_default())
        .add_attribute("timestamp", timestamp.to_string())
        .add_attribute("creator", info.sender.to_string());

    Ok(Response::new()
        .add_event(event)
        .add_attribute("action", "create_dst_escrow")
        .add_attribute("escrow_address", escrow_address))
}

// Alternative function to generate a more cosmos-like address
// DELETED.
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

// Params: CrossChainOrder, ResolverDecision
// Called by Step. 1 in CrossChainResolver.ts
pub fn create_src_escrow2() {}

// Params: CrossChainOrder, ResolverDecision
// Called by Step. 2 in CrossChainResolver.ts
pub fn create_dst_escrow2() {}


pub fn execute_transfer_with_permit(
    deps: DepsMut,
    env: Env,
    permit: Permit,
    from: String,
    to: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let from_addr = deps.api.addr_validate(&from)?;
    
    // Check if permit already used
    // if USED_PERMITS.has(deps.storage, (&from_addr, &permit.params.permit_name)) {
    //     return Err(ContractError::PermitAlreadyUsed {});
    // }
    
    // Check expiration
    // if let Some(expiration) = permit.params.expiration {
    //     if env.block.height >= expiration {
    //         return Err(ContractError::PermitExpired {});
    //     }
    // }
    
    // Verify nonce
    let current_nonce = NONCES.may_load(deps.storage, &from_addr)?.unwrap_or(0);
    // if permit.params.nonce != current_nonce {
    //     return Err(ContractError::InvalidNonce {});
    // }
    
    // Verify signature
    // verify_permit_signature(&env, &permit, &from_addr)?;
    
    // Mark permit as used and increment nonce
    USED_PERMITS.save(deps.storage, (&from_addr, &permit.params.permit_name), &true)?;
    NONCES.save(deps.storage, &from_addr, &(current_nonce + 1))?;
    
    // Execute transfer on CW20 contract
    let transfer_msg = Cw20ExecuteMsg::TransferFrom {
        owner: from,
        recipient: to.clone(),
        amount,
    };
    
    let wasm_msg = WasmMsg::Execute {
        contract_addr: config.cw20_contract.to_string(),
        msg: to_binary(&transfer_msg)?,
        funds: vec![],
    };
    
    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(wasm_msg))
        .add_attributes(vec![
            attr("action", "transfer_with_permit"),
            attr("from", from_addr),
            attr("to", to),
            attr("amount", amount),
        ]))
}