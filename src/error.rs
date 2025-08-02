use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Escrow already exists for this address")]
    EscrowAlreadyExists {},

    #[error("Invalid timestamp")]
    InvalidTimestamp {},

    #[error("Escrow not found")]
    EscrowNotFound {},

    #[error("Permit already used")]
    PermitAlreadyUsed {},

    #[error("Permit expired")]
    PermitExpired {},

    #[error("Invalid nonce")]
    InvalidNonce {},

    #[error("Invalid signature")]
    InvalidSignature {},

    #[error("Invalid signature length")]
    InvalidSignatureLength {
        length: usize
    },
}
