use cosmwasm_std::{StdError, Uint128};
use cw_utils::{Expiration, Scheduled};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("Disabled")]
    Disabled {},

    #[error("InvalidTokenReplyId")]
    InvalidTokenReplyId {},
    
    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("TooBigRoyalties: {a} + {b} > {c}")]
    TooBigRoyalties{a: u32, b: u32, c: u32},

    #[error("Royalty must bigger than 2.5%")]
    MustBigger25 {},

    #[error("Still Locked")]
    StillLocked {},

    #[error("Not Created Unstaking")]
    NotCreatedUnstaking {},


    #[error("InvalidInput")]
    InvalidInput {},

    #[error("Still in Lock period")]
    StillInLock { },

    #[error("Not FOT or gFOT token")]
    UnacceptableToken {},

    #[error("Not enough gFOT")]
    NotEnoughgFOT {},

    #[error("No Reward")]
    NoReward {},

    #[error("No Staked")]
    NoStaked {},

    #[error("Not enough bFOT, needs {bfot_accept_amount}")]
    NotEnoughbFOT { bfot_accept_amount:Uint128 },

    #[error("Not enough FOT")]
    NotEnoughFOT { },

    #[error("Already claimed")]
    Claimed {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("Map2List failed")]
    Map2ListFailed {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Airdrop stage {stage} expired at {expiration}")]
    StageExpired { stage: u8, expiration: Expiration },

    #[error("Airdrop stage {stage} begins at {start}")]
    StageNotBegun { stage: u8, start: Scheduled },

    #[error("Count {count}")]
    Count { count: u64 },
}
