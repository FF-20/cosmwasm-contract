use cosmwasm_std::StdError;
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
}