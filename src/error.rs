use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput(String),

    #[error("Already Executed")]
    AlreadyExecuted {},

    #[error("Proposal does not exist")]
    ProposalDoesNotExist {},

    #[error("Insufficient funds")]
    InsufficientFunds {},

    #[error("Member already voted")]
    MemberAlreadyVoted {},
}