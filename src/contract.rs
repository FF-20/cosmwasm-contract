use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use crate::error::ContractError;
use crate::execute;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query;

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateDstEscrow {
            escrow_address,  // Added escrow_address parameter
            immutables,
            timestamp,
        } => execute::create_dst_escrow(deps, env, info, escrow_address, immutables, timestamp),
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetDstEscrow { escrow_address } => {  // Changed from escrow_key
            to_json_binary(&query::query_dst_escrow(deps, escrow_address)?)
        }
        QueryMsg::ListDstEscrows { start_after, limit } => {
            to_json_binary(&query::query_all_dst_escrows(deps, start_after, limit)?)
        }
    }
}

#[entry_point]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Uint128};

    use crate::msg::{DstEscrowResponse, ExecuteMsg};
    use crate::state::Immutables;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // We can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_create_dst_escrow() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        // First instantiate the contract
        let instantiate_msg = InstantiateMsg {};
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let immutables = Immutables {
            order_hash: "0x1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890".to_string(),
            maker: cosmwasm_std::Addr::unchecked("maker_address"),
            taker: cosmwasm_std::Addr::unchecked("taker_address"),
            token: cosmwasm_std::Addr::unchecked("token_address"),
            amount: Uint128::from(1000u128),
            safety_deposit: Uint128::from(100u128),
            timelocks: Uint128::from(3600u128),
        };

        let timestamp = Uint128::from(env.block.time.seconds() + 3600);

        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address: "cosmos1test".to_string(),  // User-provided address
            immutables: immutables.clone(),
            timestamp,
        };

        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Check that event was emitted
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, crate::state::EVENT_TYPE_DST_ESCROW_CREATED);

        // Check that escrow was stored by querying
        let escrow_address = "cosmos1test".to_string();
        let query_msg = QueryMsg::GetDstEscrow {
            escrow_address: escrow_address.clone(),  // Changed from escrow_key
        };

        let query_res = query(deps.as_ref(), env, query_msg).unwrap();
        let escrow_response: DstEscrowResponse = cosmwasm_std::from_binary(&query_res).unwrap();

        assert_eq!(escrow_response.escrow_address, escrow_address);  // Changed from escrow_key
        assert_eq!(escrow_response.immutables, Some(immutables));
    }

    #[test]
    fn test_invalid_timestamp() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        // First instantiate the contract
        let instantiate_msg = InstantiateMsg {};
        instantiate(deps.as_mut(), env.clone(), info.clone(), instantiate_msg).unwrap();

        let immutables = Immutables {
            order_hash: "0x1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890".to_string(),
            maker: cosmwasm_std::Addr::unchecked("maker_address"),
            taker: cosmwasm_std::Addr::unchecked("taker_address"),
            token: cosmwasm_std::Addr::unchecked("token_address"),
            amount: Uint128::from(1000u128),
            safety_deposit: Uint128::from(100u128),
            timelocks: Uint128::from(3600u128),
        };

        // Use past timestamp
        let timestamp = Uint128::from(env.block.time.seconds() - 3600);

        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address: "cosmos1test".to_string(),  // User-provided address
            immutables,
            timestamp,
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::InvalidTimestamp {}));
    }
}