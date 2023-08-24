use cosmwasm_std::{
    to_binary,  Response, StdResult, Uint128, Coin, BankMsg,
    WasmMsg, WasmQuery, QueryRequest, Addr, Storage, CosmosMsg,  QuerierWrapper, BalanceResponse as NativeBalanceResponse, BankQuery
};
use cw20::{Balance, Cw20ExecuteMsg, Denom, BalanceResponse as CW20BalanceResponse, Cw20QueryMsg};
use crate::error::ContractError;
use crate::state::CONFIG;

pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 10;
pub const MAX_ORDER: u64 = 10;

pub fn multiple() -> Uint128 { Uint128::from(100u128) }
pub fn decimal() -> Uint128 { Uint128::from(1000000u128) }

pub fn check_enabled(
    storage: &mut dyn Storage,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(storage)?;
    if !cfg.enabled {
        return Err(ContractError::Disabled {})
    }
    Ok(Response::new().add_attribute("action", "check_enabled"))
}

pub fn check_owner(
    storage: &mut dyn Storage,
    address: Addr
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(storage)?;
    
    if address != cfg.owner {
        return Err(ContractError::Unauthorized {})
    }
    Ok(Response::new().add_attribute("action", "check_owner"))
}

pub fn execute_update_owner(
    storage: &mut dyn Storage,
    address: Addr,
    owner: Addr,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(storage, address)?;
    
    CONFIG.update(storage, |mut exists| -> StdResult<_> {
        exists.owner = owner.clone();
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config").add_attribute("owner", owner.clone()))
}

pub fn execute_update_enabled (
    storage: &mut dyn Storage,
    address: Addr,
    enabled: bool
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(storage, address)?;
    
    CONFIG.update(storage, |mut exists| -> StdResult<_> {
        exists.enabled = enabled;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_enabled"))
}

pub fn get_amount_of_denom(
    balance: Balance,
    denom: Denom
) -> Result<Uint128, ContractError> {

    match denom.clone() {
        Denom::Native(native_str) => {
            match balance {
                Balance::Native(native_balance) => {
                    let zero_coin = &Coin {
                        denom: String::from("empty"),
                        amount: Uint128::zero()
                    };
                    let (_index, coin) =native_balance.0.iter().enumerate().find(|(_i, c)| c.denom == native_str)
                    .unwrap_or((0, zero_coin));

                    if coin.amount == Uint128::zero() {
                        return Err(ContractError::NativeInputZero {});
                    }
                    return Ok(coin.amount);
                },
                Balance::Cw20(_) => {
                    return Err(ContractError::TokenTypeMismatch {});
                }
            }
        },
        Denom::Cw20(cw20_address) => {
            match balance {
                Balance::Native(_) => {
                    return Err(ContractError::TokenTypeMismatch {});
                },
                Balance::Cw20(token) => {
                    if cw20_address != token.address {
                        return Err(ContractError::TokenTypeMismatch {});
                    }
                    if token.amount == Uint128::zero() {
                        return Err(ContractError::Cw20InputZero {});
                    }
                    return Ok(token.amount);
                }
            }
        }
    }
}


pub fn transfer_token_message(
    denom: Denom,
    amount: Uint128,
    receiver: Addr
) -> Result<CosmosMsg, ContractError> {

    match denom.clone() {
        Denom::Native(native_str) => {
            return Ok(BankMsg::Send {
                to_address: receiver.clone().into(),
                amount: vec![Coin{
                    denom: native_str,
                    amount
                }]
            }.into());
        },
        Denom::Cw20(cw20_address) => {
            return Ok(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: receiver.clone().into(),
                    amount
                })?,
            }));
        }
    }
}


pub fn get_token_amount(
    querier: QuerierWrapper,
    denom: Denom,
    contract_addr: Addr
) -> Result<Uint128, ContractError> {

    match denom.clone() {
        Denom::Native(native_str) => {
            let native_response: NativeBalanceResponse = querier.query(&QueryRequest::Bank(BankQuery::Balance {
                address: contract_addr.clone().into(),
                denom: native_str
            }))?;
            return Ok(native_response.amount.amount);
        },
        Denom::Cw20(cw20_address) => {
            let balance_response: CW20BalanceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: cw20_address.clone().into(),
                msg: to_binary(&Cw20QueryMsg::Balance {address: contract_addr.clone().into()})?,
            }))?;
            return Ok(balance_response.balance);
        }
    }
}
