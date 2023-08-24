use cosmwasm_std::{
    to_binary,  Response, StdResult, Uint128, Coin, BankMsg,
    WasmMsg, WasmQuery, QueryRequest, Addr, Storage, CosmosMsg,  QuerierWrapper, BalanceResponse as NativeBalanceResponse, BankQuery
};
use cw20::{Balance, Cw20ExecuteMsg, Denom, BalanceResponse as CW20BalanceResponse, Cw20QueryMsg};
use crate::error::ContractError;
use crate::state::CONFIG;
use crate::msg::Royalty;
use wasmswap::msg::{ExecuteMsg as WasmswapExecuteMsg, QueryMsg as WasmswapQueryMsg, Token1ForToken2PriceResponse, Token2ForToken1PriceResponse, InfoResponse as WasmswapInfoResponse, TokenSelect};

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

pub fn execute_update_royalties (
    storage: &mut dyn Storage,
    address: Addr,
    maximum_royalty_fee: u32,
    royalties: Vec<Royalty>
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(storage, address)?;

    let mut sum = 0;
    for item in royalties.clone() {
        sum += item.rate;
    }

    if sum > maximum_royalty_fee {
        return Err(crate::ContractError::ExceedsMaximumRoyaltyFee {});
    }
    
    CONFIG.update(storage, |mut exists| -> StdResult<_> {
        exists.maximum_royalty_fee = maximum_royalty_fee;
        exists.royalties = royalties;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_royalties"))
}

pub fn check_token_and_pool (
    querier: QuerierWrapper,
    denom: Denom,
    pool_address: Addr,
) -> Result<bool, ContractError> {
    let pool_info_response: WasmswapInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.clone().into(),
        msg: to_binary(&WasmswapQueryMsg::Info {})?,
    }))?;

    if denom != pool_info_response.token1_denom && denom != pool_info_response.token2_denom {
        return Err(ContractError::PoolAndTokenMismatch{});
    }

    if denom == pool_info_response.token1_denom {
        return Ok(true);
    }

    if denom == pool_info_response.token2_denom {
        return Ok(false);
    }
    return Err(ContractError::PoolAndTokenMismatch{});
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

pub fn get_swap_amount_and_denom_and_message(
    querier: QuerierWrapper,
    pool_address: Addr,
    denom: Denom,
    amount: Uint128,
) -> Result<(Uint128, Denom, Vec<CosmosMsg>), ContractError> {

    let pool_info_response: WasmswapInfoResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.clone().into(),
        msg: to_binary(&WasmswapQueryMsg::Info {})?,
    }))?;

    if denom != pool_info_response.token1_denom && denom != pool_info_response.token2_denom {
        return Err(ContractError::PoolAndTokenMismatch{});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    let swap_amount;
    let other_denom: Denom;
    if denom == pool_info_response.token1_denom {
        let token2_price_response: Token1ForToken2PriceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_address.clone().into(),
            msg: to_binary(&WasmswapQueryMsg::Token1ForToken2Price {
                token1_amount: amount
            })?,
        }))?;

        other_denom = pool_info_response.token2_denom;
        swap_amount = token2_price_response.token2_amount;
        let messages_swap = swap_token_messages(denom, TokenSelect::Token1, amount, swap_amount, pool_address.clone())?;
        for i in 0..messages_swap.len() {
            messages.push(messages_swap[i].clone());
        }

    } else {
        let token1_price_response: Token2ForToken1PriceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_address.clone().into(),
            msg: to_binary(&WasmswapQueryMsg::Token2ForToken1Price {
                token2_amount: amount
            })?,
        }))?;

        other_denom = pool_info_response.token1_denom;
        swap_amount = token1_price_response.token1_amount;

        let messages_swap = swap_token_messages(denom, TokenSelect::Token2, amount, swap_amount, pool_address.clone())?;
        for i in 0..messages_swap.len() {
            messages.push(messages_swap[i].clone());
        }
    }
    Ok((swap_amount, other_denom, messages))
}


pub fn swap_token_messages(
    denom: Denom,
    input_token: TokenSelect,
    input_amount: Uint128,
    min_output: Uint128,
    pool_address: Addr
) -> Result<Vec<CosmosMsg>, ContractError> {

    let mut messages: Vec<CosmosMsg> = vec![];
    match denom.clone() {
        Denom::Native(native_str) => {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pool_address.clone().into(),
                funds: vec![Coin {
                    denom: native_str,
                    amount: input_amount
                }],
                msg: to_binary(&WasmswapExecuteMsg::Swap {
                    input_token,
                    input_amount,
                    min_output,
                    expiration: None
                })?,
            }));

        },
        Denom::Cw20(cw20_address) => {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cw20_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
                    spender: pool_address.clone().into(),
                    amount: input_amount,
                    expires: None
                })?,
            }));
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: pool_address.clone().into(),
                funds: vec![],
                msg: to_binary(&WasmswapExecuteMsg::Swap {
                    input_token,
                    input_amount,
                    min_output,
                    expiration: None
                })?,
            }));
        }
    }
    return Ok(messages);
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
