#[cfg(not(feature = "library"))]
use crate::ContractError;
use crate::state::{Config, CONFIG, STAKING};
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, ReplyOn, Response, Api,
    StdResult, SubMsg, Uint128, WasmMsg, Coin, from_binary, BankMsg, QueryRequest, WasmQuery, Storage, Order
};
use cw2::set_contract_version;
use cw721::{
    OwnerOfResponse
    
};
use cw20::Denom;

use cw2::{get_contract_version};
use cw721::Cw721ReceiveMsg;
use cw_storage_plus::Bound;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::InstantiateMsg as Cw721InstantiateMsg, Extension, 
    msg::MintMsg, msg::BatchMintMsg, msg::QueryMsg as Cw721QueryMsg,  msg::EditMsg
};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, MigrateMsg, NftReceiveMsg, StakingInfo};
use cw_utils::{Expiration, Scheduled};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg, Cw20CoinVerified, Balance};
use cw_utils::parse_reply_instantiate_data;
use sha2::Digest;
use std::convert::TryInto;

use crate::util;
use marble_collection::msg::{InstantiateMsg as CollectionInstantiateMsg, ExecuteMsg as CollectionExecuteMsg, QueryMsg as CollectionQueryMsg, ConfigResponse as CollectionConfigResponse};

// version info for migration info
const CONTRACT_NAME: &str = "nftstaking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const INSTANTIATE_TOKEN_REPLY_ID: u64 = 1;


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, crate::ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;


    let config = Config {
        owner: info.sender.clone(),
        collection_address: msg.collection_address.clone(),
        cw20_address: msg.cw20_address.clone(),
        daily_reward: msg.daily_reward.clone(),
        interval: msg.interval,
        lock_time: msg.lock_time,
        enabled: true
    };

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
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
        ExecuteMsg::UpdateConfig { cw20_address, daily_reward, interval, lock_time } => execute_update_config(deps.storage, info.sender, cw20_address, daily_reward, interval, lock_time),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, env, info, msg),
        ExecuteMsg::Claim { } => execute_claim(deps, env, info),
        ExecuteMsg::CreateUnstake { } => execute_create_unstake(deps, env, info),
        ExecuteMsg::FetchUnstake { } => execute_fetch_unstake(deps, env, info),
        ExecuteMsg::WithdrawId { token_id } => execute_withdraw_id(deps, env, info, token_id),
    }
}

pub fn execute_update_config (
    storage: &mut dyn Storage,
    address: Addr,
    cw20_address: Addr,
    daily_reward: Uint128,
    interval: u64,
    lock_time: u64
) -> Result<Response, ContractError> {
    // authorize owner
    util::check_owner(storage, address)?;
    
    CONFIG.update(storage, |mut exists| -> StdResult<_> {
        exists.cw20_address = cw20_address;
        exists.daily_reward = daily_reward;
        exists.interval = interval;
        exists.lock_time = lock_time;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

fn update_unclaimed_amount(
    storage: &mut dyn Storage,
    env: Env,
    address: Addr
) -> Result<Response, ContractError> {
    
    let cfg = CONFIG.load(storage)?;
    let mut record = STAKING.load(storage, address.clone())?;
    if record.create_unstake_timestamp == 0u64 {
        record.unclaimed_amount += Uint128::from((env.block.time.seconds() / cfg.interval - record.last_timestamp / cfg.interval) * (record.token_ids.len() as u64)) * cfg.daily_reward;
        record.last_timestamp = env.block.time.seconds();

        STAKING.save(storage, address.clone(), &record)?;
    }
    
    Ok(Response::new())
}

pub fn execute_receive_nft(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo, 
    wrapper: Cw721ReceiveMsg
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    let collection_response: CollectionConfigResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cfg.collection_address.clone().into(),
        msg: to_binary(&CollectionQueryMsg::GetConfig {})?,
    }))?;
    let cw721_address = collection_response.cw721_address.unwrap();

    if info.sender.clone() != cw721_address.clone() {
        return Err(crate::ContractError::InvalidCw721Token {})
    }

    let token_id = wrapper.token_id.clone();
    let user_addr = deps.api.addr_validate(wrapper.sender.as_str())?;

    let msg: NftReceiveMsg = from_binary(&wrapper.msg)?;

    match msg {
        NftReceiveMsg::Stake {} => {
            let mut record = StakingInfo {
                address: user_addr.clone(),
                token_ids: vec![token_id.clone()],
                claimed_amount: Uint128::zero(),
                unclaimed_amount: Uint128::zero(),
                claimed_timestamp: env.block.time.seconds(),
                create_unstake_timestamp: 0u64,
                last_timestamp: env.block.time.seconds()
            };

            if STAKING.has(deps.storage, user_addr.clone()) {
                update_unclaimed_amount(deps.storage, env.clone(), user_addr.clone())?;
                record = STAKING.load(deps.storage, user_addr.clone())?;
                let mut list = record.token_ids.clone();
                list.push(token_id.clone());
                record.token_ids = list;
            }
            STAKING.save(deps.storage, user_addr.clone(), &record);
            
            Ok(Response::new()
                .add_attribute("action", "execute_receive")
                .add_attribute("token_id", token_id.clone())
            )
        }
    }
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;

    if !STAKING.has(deps.storage, info.sender.clone()) {
        return Err(crate::ContractError::NotStaked {  });
    }
    let cfg = CONFIG.load(deps.storage)?;

    update_unclaimed_amount(deps.storage, env.clone(), info.sender.clone())?;

    let mut staking_info = STAKING.load(deps.storage, info.sender.clone())?;

    if staking_info.unclaimed_amount == Uint128::zero() {
        return Err(crate::ContractError::NoReward {  });
    }

    if util::get_token_amount(deps.querier, Denom::Cw20(cfg.cw20_address.clone()), env.clone().contract.address.clone())? < staking_info.unclaimed_amount {
        return Err(crate::ContractError::InsufficientCw20 {  });
    }

    let mut reward_msg = util::transfer_token_message(Denom::Cw20(cfg.cw20_address.clone()), staking_info.unclaimed_amount, info.sender.clone())?;
    let amount = staking_info.unclaimed_amount;
    staking_info.claimed_amount += staking_info.unclaimed_amount;
    staking_info.unclaimed_amount = Uint128::zero();

    staking_info.claimed_timestamp = env.clone().block.time.seconds();

    STAKING.save(deps.storage, info.sender.clone(), &staking_info);


    Ok(Response::new()
        .add_message(reward_msg)
        .add_attribute("action", "action_claim")
        .add_attribute("address", info.sender.clone().to_string())
        .add_attribute("claimed_amount", amount)
    )
}

pub fn execute_create_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;
    update_unclaimed_amount(deps.storage, env.clone(), info.sender.clone())?;

    let mut staking_info = STAKING.load(deps.storage, info.sender.clone())?;
    staking_info.create_unstake_timestamp = env.block.time.seconds();
    
    STAKING.save(deps.storage, info.sender.clone(), &staking_info)?;


    Ok(Response::new()
        .add_attribute("action", "create_unstake")
    )
}


pub fn execute_fetch_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;
    update_unclaimed_amount(deps.storage, env.clone(), info.sender.clone())?;

    let mut staking_info = STAKING.load(deps.storage, info.sender.clone())?;

    if staking_info.create_unstake_timestamp == 0u64 {
        return Err(ContractError::CreateUnstakeFirst {});
    }

    if env.block.time.seconds() < cfg.lock_time + staking_info.create_unstake_timestamp {
        return Err(ContractError::StillInLock{});
    }

    let collection_response: CollectionConfigResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cfg.collection_address.clone().into(),
        msg: to_binary(&CollectionQueryMsg::GetConfig {})?,
    }))?;
    let cw721_address = collection_response.cw721_address.unwrap();

    let mut msgs:Vec<CosmosMsg> = vec![];
    if staking_info.unclaimed_amount > Uint128::zero() {
        if util::get_token_amount(deps.querier, Denom::Cw20(cfg.cw20_address.clone()), env.clone().contract.address.clone())? < staking_info.unclaimed_amount {
            return Err(crate::ContractError::InsufficientCw20 {  });
        }
        msgs.push(util::transfer_token_message(Denom::Cw20(cfg.cw20_address.clone()), staking_info.unclaimed_amount, info.sender.clone())?);
    }
    
    for token_id in staking_info.token_ids.clone() {
        msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: cw721_address.clone().to_string(),
            msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
                token_id,
                recipient: info.sender.clone().into()
            })?,
            funds: vec![],
        }));
    }

    STAKING.remove(deps.storage, info.sender.clone());


    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "unstake")
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

    let collection_response: CollectionConfigResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: config.collection_address.clone().into(),
        msg: to_binary(&CollectionQueryMsg::GetConfig {})?,
    }))?;
    let cw721_address = collection_response.cw721_address.unwrap();

    let mut messages:Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cw721_address.clone().to_string(),
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetStaking { address} => to_binary(&query_get_staking(deps, env, address)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        collection_address: config.collection_address,
        cw20_address: config.cw20_address,
        daily_reward: config.daily_reward,
        interval: config.interval,
        lock_time: config.lock_time,
        enabled: config.enabled
    })
}

fn query_get_staking(
    deps: Deps,
    env: Env,
    address: Addr,
) -> StdResult<StakingInfo> {

    let mut staking_info = STAKING.load(deps.storage, address.clone())?;
    let cfg = CONFIG.load(deps.storage)?;

    if staking_info.create_unstake_timestamp == 0u64 {
        staking_info.unclaimed_amount += Uint128::from((env.block.time.seconds() / cfg.interval - staking_info.last_timestamp / cfg.interval) * (staking_info.token_ids.len() as u64)) * cfg.daily_reward;    
    }

    Ok(staking_info)
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
