use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    #[error("message is not in the allow list {0}")]
    DisallowedMessage(String),
    #[error("not allowed to spend fees on contract {0}")]
    DisallowedContract(String),
    #[error("decode error")]
    DecodeError(#[from] ::prost::DecodeError),
}
