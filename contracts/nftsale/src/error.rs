use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;
use cw_utils::{Expiration, Scheduled};

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Already on Sale")]
    AlreadyOnSale {},

    #[error("CannotCancelSale")]
    CannotCancelSale {},

    #[error("TooBigRoyalties: {a} + {b} > {c}")]
    TooBigRoyalties{a: u32, b: u32, c: u32},

    #[error("Royalty must bigger than 2.5%")]
    MustBigger25 {},


    #[error("InvalidSaleType")]
    InvalidSaleType {},

    #[error("DurationIncorrect")]
    DurationIncorrect {},

    #[error("NotOnSale")]
    NotOnSale {},

    #[error("NoBids")]
    NoBids {},
    
    #[error("NotSupported")]
    NotSupported {},

    #[error("NotStarted")]
    NotStarted {},

    #[error("Disabled")]
    Disabled {},

    #[error("The pool does not contain the input token")]
    PoolAndTokenMismatch {},

    #[error("Amount of the native coin inputed is zero")]
    NativeInputZero {},

    #[error("Amount of the cw20 coin inputed is zero")]
    Cw20InputZero {},

    #[error("Token type mismatch")]
    TokenTypeMismatch {},

    #[error("NotMinted")]
    NotMinted {},

    #[error("InvalidBuyParam")]
    InvalidBuyParam {},

    #[error("InvalidUserOrPrice")]
    InvalidUserOrPrice {},

    #[error("LowerThanReserved")]
    LowerThanReserved {},

    #[error("InvalidCw20Token")]
    InvalidCw20Token {},

    #[error("InvalidNativeToken")]
    InvalidNativeToken {},

    #[error("InvalidCw721Token")]
    InvalidCw721Token {},

    #[error("InvalidUnitPrice")]
    InvalidUnitPrice {},

    #[error("InvalidMaxTokens")]
    InvalidMaxTokens {},

    #[error("InvalidFirstRoyalty")]
    InvalidFirstRoyalty {},

    #[error("ExceedsMaximumRoyaltyFee")]
    ExceedsMaximumRoyaltyFee {},

    #[error("MaxTokensExceed")]
    MaxTokensExceed {},

    #[error("OnlyNativeSell")]
    OnlyNativeSell {},

    #[error("UnauthorizedTokenContract")]
    UnauthorizedTokenContract {},

    #[error("Uninitialized")]
    Uninitialized {},

    
    #[error("Cannot edit on Sale")]
    CannotEditOnSale {},

    #[error("CountNotMatch")]
    CountNotMatch {},

    #[error("WrongPaymentAmount")]
    WrongPaymentAmount {},

    #[error("InvalidTokenReplyId")]
    InvalidTokenReplyId {},

    #[error("Cw721AlreadyLinked")]
    Cw721AlreadyLinked {},

    #[error("Incorrect funds")]
    IncorrectFunds {},

    #[error("Verification failed")]
    VerificationFailed {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Insufficient Tokens")]
    InsufficientFund {},

    #[error("AlreadySold")]
    AlreadySold {},

    #[error("AlreadyExpired")]
    AlreadyExpired {},

    #[error("NotExpired")]
    NotExpired {},

    #[error("AlreadyFinished")]
    AlreadyFinished{},
    
    #[error("LowerThanPrevious")]
    LowerThanPrevious {},

    #[error("LowerPrice")]
    LowerPrice {},

    #[error("Already claimed")]
    Claimed {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("InsufficientRoyalty")]
    InsufficientRoyalty {},
}
