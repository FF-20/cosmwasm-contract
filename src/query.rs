
/// @notice Returns the current status of a swap (pending, completed, unknown)
/// @param deps - The dependencies of the contract, see cosmwasm_std::Deps
/// @param swap_id - The ID of the swap to query.
/// @return The status of the swap, see crate::msg::SwapStatusResponse
pub fn query_swap_status(deps: Deps, swap_id: String) -> StdResult<SwapStatusResponse> {
    let swap_data = SWAPS.may_load(deps.storage, swap_id.clone())?;
    
    let status = match swap_data {
        Some(data) => data.status,
        None => SwapStatus::Unknown,
    };

    Ok(SwapStatusResponse {
        swap_id,
        status,
    })
}

/// Query a specific swap by ID
pub fn query_swap(deps: Deps, swap_id: String) -> StdResult<SwapResponse> {
    let swap_data = SWAPS.may_load(deps.storage, swap_id.clone())?;

    Ok(SwapResponse {
        swap_id,
        swap_data,
    })
}

/// Query all swaps with pagination
pub fn query_all_swaps(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<SwapListResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;

    let start = start_after
        .as_deref()
        .map(|s| Bound::exclusive(s));

    let swaps: StdResult<Vec<(String, SwapData)>> = SWAPS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    Ok(SwapListResponse {
        swaps: swaps?,
    })
}/// Query a specific source escrow
pub fn query_src_escrow(deps: Deps, escrow_address: String) -> StdResult<SrcEscrowResponse> {
    let escrow_data = SRC_ESCROWS.may_load(deps.storage, escrow_address.clone())?;

    Ok(SrcEscrowResponse {
        escrow_address,
        escrow_data,
    })
}

/// Query all source escrows with pagination
pub fn query_all_src_escrows(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<SrcEscrowListResponse> {
    let limit = limit.unwrap_or(30).min(100) as usize;

    let start = start_after
        .as_deref()
        .map(|s| Bound::exclusive(s));

    let escrows: StdResult<Vec<(String, SrcEscrowData)>> = SRC_ESCROWS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    Ok(SrcEscrowListResponse {
        escrows: escrows?,
    })
}use cosmwasm_std::{Deps, Order, StdResult};
use cw_storage_plus::Bound;

use crate::msg::{DstEscrowListResponse, DstEscrowResponse, SrcEscrowResponse, SrcEscrowListResponse, SwapStatusResponse, SwapResponse, SwapListResponse};
use crate::state::{Immutables, SrcEscrowData, SwapData, SwapStatus, DST_ESCROWS, SRC_ESCROWS, SWAPS};

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
        .map(|s| Bound::exclusive(s));

    let escrows: StdResult<Vec<(String, Immutables)>> = DST_ESCROWS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    Ok(DstEscrowListResponse {
        escrows: escrows?,
    })
}