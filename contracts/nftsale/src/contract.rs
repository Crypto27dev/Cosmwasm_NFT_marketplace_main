use std::ops::Index;

#[cfg(not(feature = "library"))]
use crate::ContractError;
use crate::state::{Config, CONFIG, TOKENS};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, Api,
    StdResult, SubMsg, Uint128, WasmMsg, Coin, from_binary, BankMsg, QueryRequest, WasmQuery, Storage, Order
};
use cw2::set_contract_version;
use cw721::{
    OwnerOfResponse,
    
};
use cw20::Denom;

use cw2::{get_contract_version};
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, Extension
};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, };

use cw20::{ Balance};

use crate::util;

// version info for migration info
const CONTRACT_NAME: &str = "nftsale";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, crate::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let mut unsold_list:Vec<String> = vec![];
    

    let config = Config {
        owner: info.sender.clone(),
        price: msg.price,
        denom: msg.denom,
        total_count: 0u32,
        sold_index: 0u32,
        cw721_address: msg.cw721_address,
        enabled: true,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetToken {index} => to_binary(&query_get_token(deps, index)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        price: config.price,
        total_count: config.total_count,
        sold_index: config.sold_index,
        cw721_address: config.cw721_address,
        enabled: config.enabled,
        denom: config.denom,
    })
}

fn query_get_token(
    deps: Deps,
    index: u32
) -> StdResult<String> {
    let token_id = TOKENS.load(deps.storage, index).unwrap();
    Ok(token_id)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, crate::ContractError> {
    match msg {
        ExecuteMsg::UpdateOwner { owner } => util::execute_update_owner(deps.storage, info.sender, owner),
        ExecuteMsg::UpdateEnabled { enabled } => util::execute_update_enabled(deps.storage, info.sender, enabled),
        ExecuteMsg::SetToken {token_id} => execute_set_token(deps, token_id),
        ExecuteMsg::Buy { } => execute_buy(deps, env, info),
        ExecuteMsg::Withdraw { index } => execute_withdraw(deps, env, info, index),
        ExecuteMsg::WithdrawId { token_id } => execute_withdraw_id(deps, env, info, token_id),
    }
}

pub fn execute_set_token(
    deps: DepsMut,
    token_id: String
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    
    TOKENS.save(deps.storage, config.total_count, &token_id)?;
    config.total_count += 1;
    CONFIG.save(deps.storage, &config)?;
    
    Ok(Response::new()
        .add_attribute("action", "set_token")
        .add_attribute("token_id", token_id.to_string())
    )
}


pub fn execute_buy(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    
    if config.total_count == config.sold_index + 1 {
        return Err(ContractError::AlreadyFinished {  })
    }
    let amount = util::get_amount_of_denom(Balance::from(info.funds), Denom::Native(config.denom.clone()))?;
    if amount < config.price {
        return Err(ContractError::InsufficientFund {  })
    }

    config.sold_index += 1;
    CONFIG.save(deps.storage, &config)?;

    let token_id = TOKENS.load(deps.storage, config.sold_index)?;

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(util::transfer_token_message(Denom::Native(config.denom.clone()), amount, config.owner.clone())?);

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().to_string(),
        msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
            token_id: token_id.clone(),
            recipient: info.sender.clone().into()
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "buy")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("buyer", info.sender.clone())
    )
}


pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: u32
) -> Result<Response, crate::ContractError> {

    util::check_owner(deps.storage, info.sender.clone())?;
    let token_id = TOKENS.load(deps.storage, index)?;
    let config = CONFIG.load(deps.storage)?;

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().to_string(),
        msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
            token_id: token_id.clone(),
            recipient: config.owner.clone().into()
        })?,
        funds: vec![],
    }));


    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "withdraw")
        .add_attribute("token_id", token_id)
    )
}


pub fn execute_withdraw_id(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String
) -> Result<Response, crate::ContractError> {

    util::check_owner(deps.storage, info.sender.clone())?;
    let config = CONFIG.load(deps.storage)?;

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().to_string(),
        msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
            token_id: token_id.clone(),
            recipient: config.owner.clone().into()
        })?,
        funds: vec![],
    }));

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "withdraw_id")
        .add_attribute("token_id", token_id)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, crate::ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(crate::ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}
