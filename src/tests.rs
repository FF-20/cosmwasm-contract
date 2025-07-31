#[cfg(test)]
mod escrow_tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, Uint128, Addr};
    use crate::msg::{ExecuteMsg, QueryMsg, DstEscrowResponse};
    use crate::state::Immutables;

    #[test]
    fn test_create_dst_escrow_success() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        // Create immutables struct
        let immutables = Immutables {
            order_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            maker: Addr::unchecked("cosmos1maker123456789012345678901234567890"),
            taker: Addr::unchecked("cosmos1taker123456789012345678901234567890"),
            token: Addr::unchecked("cosmos1token123456789012345678901234567890"),
            amount: Uint128::from(1000000u128),
            safety_deposit: Uint128::from(100000u128),
            timelocks: Uint128::from(3600u128),
        };

        let escrow_address = "cosmos1escrow12345678901234567890123456789012345678".to_string();
        let timestamp = Uint128::from(env.block.time.seconds() + 7200); // 2 hours in future

        // Create the execute message
        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address: escrow_address.clone(),
            immutables: immutables.clone(),
            timestamp,
        };

        // Execute the create escrow function
        let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Verify response attributes
        assert_eq!(res.attributes.len(), 2);
        assert_eq!(res.attributes[0].key, "action");
        assert_eq!(res.attributes[0].value, "create_dst_escrow");
        assert_eq!(res.attributes[1].key, "escrow_address");
        assert_eq!(res.attributes[1].value, escrow_address);

        // Verify event was emitted
        assert_eq!(res.events.len(), 1);
        assert_eq!(res.events[0].ty, "dst_escrow_created");
        
        // Check event attributes
        let event_attrs = &res.events[0].attributes;
        assert!(event_attrs.iter().any(|attr| attr.key == "escrow_address" && attr.value == escrow_address));
        assert!(event_attrs.iter().any(|attr| attr.key == "order_hash" && attr.value == immutables.order_hash));
        assert!(event_attrs.iter().any(|attr| attr.key == "amount" && attr.value == immutables.amount.to_string()));

        // Query the created escrow to verify it was stored
        let query_msg = QueryMsg::GetDstEscrow {
            escrow_address: escrow_address.clone(),
        };

        let query_res = query(deps.as_ref(), env, query_msg).unwrap();
        let escrow_response: DstEscrowResponse = cosmwasm_std::from_binary(&query_res).unwrap();

        assert_eq!(escrow_response.escrow_address, escrow_address);
        assert_eq!(escrow_response.immutables, Some(immutables));
    }

    #[test]
    fn test_create_escrow_with_invalid_timestamp() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        let immutables = Immutables {
            order_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            maker: Addr::unchecked("cosmos1maker123456789012345678901234567890"),
            taker: Addr::unchecked("cosmos1taker123456789012345678901234567890"),
            token: Addr::unchecked("cosmos1token123456789012345678901234567890"),
            amount: Uint128::from(1000000u128),
            safety_deposit: Uint128::from(100000u128),
            timelocks: Uint128::from(3600u128),
        };

        let escrow_address = "cosmos1escrow12345678901234567890123456789012345678".to_string();
        let timestamp = Uint128::from(env.block.time.seconds() - 3600); // Past timestamp

        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address,
            immutables,
            timestamp,
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::InvalidTimestamp {}));
    }

    #[test]
    fn test_create_escrow_duplicate_address() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        let immutables = Immutables {
            order_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            maker: Addr::unchecked("cosmos1maker123456789012345678901234567890"),
            taker: Addr::unchecked("cosmos1taker123456789012345678901234567890"),
            token: Addr::unchecked("cosmos1token123456789012345678901234567890"),
            amount: Uint128::from(1000000u128),
            safety_deposit: Uint128::from(100000u128),
            timelocks: Uint128::from(3600u128),
        };

        let escrow_address = "cosmos1escrow12345678901234567890123456789012345678".to_string();
        let timestamp = Uint128::from(env.block.time.seconds() + 3600);

        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address: escrow_address.clone(),
            immutables: immutables.clone(),
            timestamp,
        };

        // First creation should succeed
        execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();

        // Second creation with same address should fail
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::EscrowAlreadyExists {}));
    }

    #[test]
    fn test_create_escrow_invalid_address_format() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        let immutables = Immutables {
            order_hash: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            hashlock: "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
            maker: Addr::unchecked("cosmos1maker123456789012345678901234567890"),
            taker: Addr::unchecked("cosmos1taker123456789012345678901234567890"),
            token: Addr::unchecked("cosmos1token123456789012345678901234567890"),
            amount: Uint128::from(1000000u128),
            safety_deposit: Uint128::from(100000u128),
            timelocks: Uint128::from(3600u128),
        };

        let invalid_address = "invalid_address_format".to_string();
        let timestamp = Uint128::from(env.block.time.seconds() + 3600);

        let msg = ExecuteMsg::CreateDstEscrow {
            escrow_address: invalid_address,
            immutables,
            timestamp,
        };

        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert!(matches!(err, ContractError::Std(_)));
    }

    // Integration test with multiple escrows
    #[test]
    fn test_create_multiple_escrows() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("creator", &coins(1000, "token"));

        // Create first escrow
        let immutables1 = Immutables {
            order_hash: "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(),
            hashlock: "0x2222222222222222222222222222222222222222222222222222222222222222".to_string(),
            maker: Addr::unchecked("cosmos1maker1"),
            taker: Addr::unchecked("cosmos1taker1"),
            token: Addr::unchecked("cosmos1token1"),
            amount: Uint128::from(1000u128),
            safety_deposit: Uint128::from(100u128),
            timelocks: Uint128::from(3600u128),
        };

        let msg1 = ExecuteMsg::CreateDstEscrow {
            escrow_address: "cosmos1escrow1".to_string(),
            immutables: immutables1,
            timestamp: Uint128::from(env.block.time.seconds() + 3600),
        };

        execute(deps.as_mut(), env.clone(), info.clone(), msg1).unwrap();

        // Create second escrow
        let immutables2 = Immutables {
            order_hash: "0x3333333333333333333333333333333333333333333333333333333333333333".to_string(),
            hashlock: "0x4444444444444444444444444444444444444444444444444444444444444444".to_string(),
            maker: Addr::unchecked("cosmos1maker2"),
            taker: Addr::unchecked("cosmos1taker2"),
            token: Addr::unchecked("cosmos1token2"),
            amount: Uint128::from(2000u128),
            safety_deposit: Uint128::from(200u128),
            timelocks: Uint128::from(7200u128),
        };

        let msg2 = ExecuteMsg::CreateDstEscrow {
            escrow_address: "cosmos1escrow2".to_string(),
            immutables: immutables2,
            timestamp: Uint128::from(env.block.time.seconds() + 7200),
        };

        execute(deps.as_mut(), env.clone(), info, msg2).unwrap();

        // Query both escrows
        let query1 = QueryMsg::GetDstEscrow {
            escrow_address: "cosmos1escrow1".to_string(),
        };
        let query2 = QueryMsg::GetDstEscrow {
            escrow_address: "cosmos1escrow2".to_string(),
        };

        let res1 = query(deps.as_ref(), env.clone(), query1).unwrap();
        let res2 = query(deps.as_ref(), env, query2).unwrap();

        let escrow1: DstEscrowResponse = cosmwasm_std::from_binary(&res1).unwrap();
        let escrow2: DstEscrowResponse = cosmwasm_std::from_binary(&res2).unwrap();

        assert!(escrow1.immutables.is_some());
        assert!(escrow2.immutables.is_some());
        assert_eq!(escrow1.escrow_address, "cosmos1escrow1");
        assert_eq!(escrow2.escrow_address, "cosmos1escrow2");
    }
}