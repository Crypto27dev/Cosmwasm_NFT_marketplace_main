#[cfg(not(feature = "library"))]
use crate::ContractError;
use crate::state::{Config, CONFIG, SALE};
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

use crate::constants::{ATOMPOOL, OSMOPOOL, USDCPOOL, SCRTPOOL, BLOCKATOMPOOL, BLOCKJUNOPOOL, BLOCKMARBLEPOOL, ATOMDENOM, OSMODENOM, USDCDENOM, SCRTDENOM, JUNODENOM, BLOCKADDR, MARBLEADDR};
use cw2::{get_contract_version};
use cw721::Cw721ReceiveMsg;
use cw_storage_plus::Bound;
use cw721_base::{
    msg::ExecuteMsg as Cw721ExecuteMsg, msg::InstantiateMsg as Cw721InstantiateMsg, Extension, 
    msg::MintMsg, msg::BatchMintMsg, msg::QueryMsg as Cw721QueryMsg,  msg::EditMsg
};
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, MigrateMsg, SaleType, DurationType, SaleInfo, SalesResponse, Request, NftReceiveMsg};
use cw_utils::{Expiration, Scheduled};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg, Cw20CoinVerified, Balance};
use cw_utils::parse_reply_instantiate_data;
use sha2::Digest;
use std::convert::TryInto;

use crate::util;

// version info for migration info
const CONTRACT_NAME: &str = "marble-collection";
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


    if msg.max_tokens == 0 {
        return Err(crate::ContractError::InvalidMaxTokens {});
    }

    if msg.royalties.clone().len() == 0 || msg.royalties.first().unwrap().address != msg.owner.clone() {
        return Err(crate::ContractError::InvalidFirstRoyalty {});
    }
    let mut sum = 0;
    for item in msg.royalties.clone() {
        sum += item.rate;
    }

    if sum > msg.maximum_royalty_fee {
        return Err(crate::ContractError::ExceedsMaximumRoyaltyFee {});
    }

    let config = Config {
        owner: msg.owner.clone(),
        cw721_address: None,
        max_tokens: msg.max_tokens,
        name: msg.name.clone(),
        symbol: msg.symbol.clone(),
        unused_token_id: 1,
        maximum_royalty_fee: msg.maximum_royalty_fee,
        royalties: msg.royalties,
        enabled: true,
        uri: msg.uri
    };

    CONFIG.save(deps.storage, &config)?;

    let sub_msg: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_binary(&Cw721InstantiateMsg {
                name: msg.name.clone() + " cw721_base",
                symbol: msg.symbol,
                minter: env.contract.address.to_string(),
            })?,
            funds: vec![],
            admin: None,
            label: msg.name.clone(),
        }
        .into(),
        id: INSTANTIATE_TOKEN_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new().add_submessages(sub_msg))
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, crate::ContractError> {
    let mut config: Config = CONFIG.load(deps.storage)?;

    if config.cw721_address != None {
        return Err(crate::ContractError::Cw721AlreadyLinked {});
    }

    if msg.id != INSTANTIATE_TOKEN_REPLY_ID {
        return Err(crate::ContractError::InvalidTokenReplyId {});
    }

    let reply = parse_reply_instantiate_data(msg).unwrap();
    config.cw721_address = Addr::unchecked(reply.contract_address).into();
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetSale {token_id} => to_binary(&query_get_sale(deps, token_id)?),
        QueryMsg::GetSales {start_after, limit} => to_binary(&query_get_sales(deps, start_after, limit)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: config.owner,
        cw721_address: config.cw721_address,
        max_tokens: config.max_tokens,
        name: config.name,
        symbol: config.symbol,
        unused_token_id: config.unused_token_id,
        maximum_royalty_fee: config.maximum_royalty_fee,
        royalties: config.royalties,
        uri: config.uri,
        enabled: config.enabled
    })
}


fn query_get_sale(
    deps: Deps,
    token_id: u32,
) -> StdResult<SaleInfo> {

    let sale_info = SALE.load(deps.storage, token_id.to_string())?;
    Ok(sale_info)
}
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 20;


fn map_sales(
    item: StdResult<(String, SaleInfo)>,
) -> StdResult<SaleInfo> {
    item.map(|(_id, record)| {
        record
    })
}

fn query_get_sales(
    deps: Deps,
    start_after: Option<u32>,
    limit: Option<u32>
) -> StdResult<SalesResponse> {

    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start = start_after.map(|str| Bound::exclusive(str.to_string()));
    
    let sales:StdResult<Vec<_>> = SALE
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| map_sales(item))
        .collect();

    Ok(SalesResponse {
        list: sales?
    })
    
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
        ExecuteMsg::UpdateRoyalties { maximum_royalty_fee, royalties } => util::execute_update_royalties(deps.storage, info.sender, maximum_royalty_fee, royalties),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, info, msg),
        ExecuteMsg::AcceptSale { token_id } => {
            execute_accept_sale(deps, info, token_id)
        },
        ExecuteMsg::CancelSale { token_id } => {
            execute_cancel_sale(deps, info, token_id)
        },
        ExecuteMsg::Mint{ uri, extension } => {
            execute_mint(deps, env, info, uri, extension)
        },
        ExecuteMsg::BatchMint{ uri, extension, owner} => {
            execute_batch_mint(deps, env, info, uri, extension, owner)
        },

        ExecuteMsg::Propose{token_id, denom} => execute_propose(deps, env, info, token_id, denom),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        
        
        ExecuteMsg::ChangeContract {    //Change the holding CW721 contract address
            cw721_address
        } => execute_change_contract(deps, info, cw721_address),
        ExecuteMsg::ChangeCw721Owner {       //Change the owner of Cw721 contract
            owner
        } => execute_change_cw721_owner(deps, info, owner),
        ExecuteMsg::UpdateUnusedTokenId {
            token_id
        } => execute_update_unused_token_id(deps, info, token_id),
        ExecuteMsg::EditSale {
            token_id,
            sale_type,
            duration_type,
            initial_price,
            reserve_price,
            denom
        } => execute_edit_sale(deps, info, token_id, sale_type, duration_type, initial_price, reserve_price, denom),
        ExecuteMsg::CancelPropose { token_id } => execute_cancel_propose(deps, info, token_id)

    }
}


pub fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    uri: String,
    extension: Extension
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    
    if config.cw721_address == None {
        return Err(crate::ContractError::Uninitialized {});
    }

    if config.unused_token_id >= config.max_tokens {
        return Err(crate::ContractError::MaxTokensExceed {});
    }

    let mint_msg = Cw721ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: config.unused_token_id.to_string(),
        owner: info.sender.clone().into(),
        token_uri: uri.clone().into(),
        extension: extension.clone(),
    });

    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().unwrap().to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });

    config.unused_token_id += 1;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_message(callback))
}


pub fn execute_batch_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    uri: Vec<String>,
    extension: Vec<Extension>,
    owner: Vec<String>
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(crate::ContractError::Unauthorized {});
    }

    if uri.len() != extension.len() {
        return Err(crate::ContractError::CountNotMatch {});
    }

    if config.cw721_address == None {
        return Err(crate::ContractError::Uninitialized {});
    }

    if config.unused_token_id >= config.max_tokens {
        return Err(crate::ContractError::MaxTokensExceed {});
    }

    let count = uri.len();
    let mut token_id:Vec<String> = vec![];
    for _i in 0..count {
        token_id.push(config.unused_token_id.to_string());
        config.unused_token_id += 1;
    }
    
    let mint_msg = Cw721ExecuteMsg::BatchMint(BatchMintMsg::<Extension> {
        token_id,
        owner,
        token_uri: uri,
        extension: extension.clone(),
    });

    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().unwrap().to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_message(callback))
}


pub fn execute_receive_nft(
    deps: DepsMut, 
    info: MessageInfo, 
    wrapper: Cw721ReceiveMsg
) -> Result<Response, crate::ContractError> {
    util::check_enabled(deps.storage)?;
    let cfg = CONFIG.load(deps.storage)?;

    if info.sender.clone() != cfg.cw721_address.clone().unwrap() {
        return Err(crate::ContractError::InvalidCw721Token {})
    }

    let token_id = wrapper.token_id.clone();
    let user_addr = deps.api.addr_validate(wrapper.sender.as_str())?;

    let msg: NftReceiveMsg = from_binary(&wrapper.msg)?;

    if SALE.has(deps.storage, token_id.clone()) {
        return Err(crate::ContractError::AlreadyOnSale {});
    }

    match msg {
        NftReceiveMsg::StartSale {sale_type, duration_type, initial_price, reserve_price, denom} => {
            if sale_type == SaleType::Fixed && duration_type != DurationType::Fixed {
                return Err(crate::ContractError::InvalidSaleType {});
            }
            
            match duration_type.clone() {
                DurationType::Time(start, end) => {
                    if start >= end {
                        return Err(crate::ContractError::DurationIncorrect {});
                    }
                },
                DurationType::Fixed => {},
                DurationType::Bid(_count) => {}
            }
        
            let info = SaleInfo {
                token_id: token_id.parse().unwrap(),
                provider: user_addr.clone(),
                sale_type,
                duration_type,
                initial_price,
                reserve_price,
                requests: vec![],
                denom,
                can_accept: false
            };
            
            SALE.save(deps.storage, token_id.clone(), &info)?;
            Ok(Response::new()
                .add_attribute("action", "start_sale")
                .add_attribute("token_id", token_id.clone())
                .add_attribute("initial_price", initial_price)
                .add_attribute("reserve_price", reserve_price)
            )
        }
    }
}

pub fn execute_accept_sale(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;

    if !SALE.has(deps.storage, token_id.to_string()) {
        return Err(crate::ContractError::NotOnSale {});
    }
    
    let sale_info = SALE.load(deps.storage, token_id.to_string())?;
    
    if sale_info.provider != info.sender {
        return Err(crate::ContractError::Unauthorized {  });
    }
    
    if sale_info.requests.len() == 0 {
        return Err(crate::ContractError::NoBids {});
    }
    
    let list = sale_info.requests.clone();
    let len = sale_info.requests.len();
    let sell_request = list.get(len - 1).unwrap();
    //Add NFT send msg
    let mut msgs = sell_nft_messages(deps.storage, deps.api, sell_request.address.clone(), sell_request.price.clone(), sale_info.clone())?;

    //Add return fund msg
    for i in 0..len - 1 {
        let request = list.get(i).unwrap();
        msgs.push(util::transfer_token_message(sale_info.denom.clone(), request.price, request.address.clone())?);
    }
    
    SALE.remove(deps.storage, token_id.to_string());

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "accept_sale")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("address", sell_request.address.clone().to_string())
        .add_attribute("price", sell_request.price)
    )
}

pub fn execute_cancel_sale(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;

    if !SALE.has(deps.storage, token_id.to_string()) {
        return Err(crate::ContractError::NotOnSale {});
    }
    
    let sale_info = SALE.load(deps.storage, token_id.to_string())?;

    if sale_info.provider != info.sender {
        return Err(crate::ContractError::Unauthorized {  });
    }

    if sale_info.can_accept {
        return Err(crate::ContractError::CannotCancelSale {  });
    }

    let cfg = CONFIG.load(deps.storage)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.cw721_address.clone().unwrap().to_string(),
        funds: vec![],
        msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
            recipient: info.sender.clone().into(),
            token_id: sale_info.token_id.to_string()
        })?,
    }));

    let list = sale_info.requests.clone();
    //Add return fund msg
    for i in 0..list.len() {
        let request = list.get(i).unwrap();
        msgs.push(util::transfer_token_message(sale_info.denom.clone(), request.price, request.address.clone())?);
    }

    SALE.remove(deps.storage, token_id.to_string());

    Ok(Response::new()
        .add_messages(msgs)
        .add_attribute("action", "cancel_sale")
        .add_attribute("token_id", token_id.to_string())
    )
}


pub fn execute_propose(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: u32,
    denom: String 
) -> Result<Response, crate::ContractError> {

    let sale_info = SALE.load(deps.storage, token_id.to_string())?;
    if sale_info.denom != Denom::Native(denom.clone()) {
        return Err(crate::ContractError::InvalidNativeToken {})
    }

    let amount = util::get_amount_of_denom(Balance::from(info.funds), Denom::Native(denom.clone()))?;

    handle_propose(deps, env, token_id, info.sender.clone(), amount)
    
}

pub fn execute_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo, 
    wrapper: Cw20ReceiveMsg
) -> Result<Response, crate::ContractError> {

    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    
    let user_addr = deps.api.addr_validate(&wrapper.sender)?;
    let cw20_amount = wrapper.amount;
    
    match msg {
        ReceiveMsg::Propose { token_id } => {

            let sale_info = SALE.load(deps.storage, token_id.to_string())?;
            if sale_info.denom != Denom::Cw20(info.sender.clone()) {
                return Err(crate::ContractError::InvalidCw20Token {})
            }
            handle_propose(deps, env, token_id, user_addr.clone(), cw20_amount)
        }
    }
}

pub fn handle_propose(
    deps: DepsMut,
    env: Env,
    token_id: u32,
    address: Addr, 
    price: Uint128
) -> Result<Response, crate::ContractError> {

    util::check_enabled(deps.storage)?;
    if !SALE.has(deps.storage, token_id.to_string()) {
        return Err(crate::ContractError::NotOnSale {});
    }
    let mut sale_info = SALE.load(deps.storage, token_id.to_string())?;

    match sale_info.duration_type.clone() {
        DurationType::Fixed => {

        }
        DurationType::Time(start, end) => {
            if env.block.time.seconds() > end {
                return Err(crate::ContractError::AlreadyExpired{})
            }
            if env.block.time.seconds() < start {
                return Err(crate::ContractError::NotStarted{})
            }
        },
        DurationType::Bid(threshold) => {
            if sale_info.requests.len() as u32 > threshold {
                return Err(crate::ContractError::AlreadyExpired{})
            }
        },
    }

    let mut list = sale_info.requests.clone();
    
    if sale_info.sale_type == SaleType::Fixed {
        if sale_info.initial_price > price {
            return Err(crate::ContractError::LowerPrice{})
        }
    } else if sale_info.sale_type == SaleType::Auction {
        
        if list.len() == 0 && price < sale_info.initial_price || list.len() > 0 && list[list.len() - 1].price >= price {
            return Err(crate::ContractError::LowerThanPrevious {})
        }
    }
    // let mut lastitem = Request {
    //     address: address.clone(),
    //     price: Uint128::zero()
    // };
    // if list.len() > 0 {
    //     let len = list.len();
    //     lastitem = list[len - 1].clone();
    // }
    list.push(Request {
        address: address.clone(),
        price
    });
    
    sale_info.requests = list.clone();

    if sale_info.sale_type == SaleType::Auction && price >= sale_info.reserve_price {
        sale_info.can_accept = true;
    }

    SALE.save(deps.storage, token_id.to_string(), &sale_info)?;

    //Handle Fixed
    if sale_info.sale_type == SaleType::Fixed {
        //send NFT messages
        let msgs = sell_nft_messages(deps.storage, deps.api, address.clone(), price, sale_info)?;
        //Remove Entry
        SALE.remove(deps.storage, token_id.to_string());

        return Ok(Response::new()
            .add_messages(msgs)
            .add_attribute("action", "fixed_sell")
            .add_attribute("address", address.clone())
            .add_attribute("token_id", token_id.to_string())
            .add_attribute("price", price)
        );

    } else {
        let mut msgs:Vec<CosmosMsg> = vec![];
        // if list.len() > 1 {
        //     msgs.push(util::transfer_token_message(sale_info.denom.clone(), lastitem.price, lastitem.address.clone())?);
        // }
        
        Ok(Response::new()
            .add_messages(msgs)
            .add_attribute("action", "propose")
            .add_attribute("address", address.clone())
            .add_attribute("token_id", token_id.to_string())
            .add_attribute("price", price)
        )    
    }
}

const MULTIPLY:u32 = 1000000u32;

pub fn sell_nft_messages (
    storage: &mut dyn Storage,
    api: &dyn Api,
    recipient: Addr,
    amount: Uint128,
    sale_info: SaleInfo
) -> Result<Vec<CosmosMsg>, crate::ContractError> {
    let cfg = CONFIG.load(storage)?;
    let mut list:Vec<Request> = vec![];

    let mut provider_amount = amount;

    for item in cfg.royalties {
        let amount = amount * Uint128::from(item.rate) / Uint128::from(MULTIPLY);
        provider_amount -= amount;
        list.push(Request { address: item.address.clone(), price: amount });
    }
    
    list.push(Request { address: sale_info.provider.clone(), price: provider_amount });

    let mut msgs: Vec<CosmosMsg> = vec![];
    msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.cw721_address.clone().unwrap().to_string(),
        funds: vec![],
        msg: to_binary(&Cw721ExecuteMsg::<Extension>::TransferNft {
            recipient: recipient.clone().into(),
            token_id: sale_info.token_id.to_string()
        })?,
    }));

    for item in list {
        if item.price == Uint128::zero() {
            continue;
        }
        msgs.push(util::transfer_token_message(sale_info.denom.clone(), item.price, item.address.clone())?);
    }

    Ok(msgs)
}


pub fn execute_edit_sale(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32,
    sale_type: SaleType,
    duration_type: DurationType,
    initial_price: Uint128,
    reserve_price: Uint128,
    denom: Denom 
) -> Result<Response, crate::ContractError> {

    let mut sale_info = SALE.load(deps.storage, token_id.to_string())?;
    if sale_info.provider != info.sender.clone() {
        return Err(crate::ContractError::Unauthorized {  });
    }

    if sale_info.requests.len() > 0 {
        return Err(crate::ContractError::AlreadyOnSale {  });
    }

    sale_info.sale_type = sale_type;
    sale_info.duration_type = duration_type;
    sale_info.initial_price = initial_price;
    sale_info.reserve_price = reserve_price;

    SALE.save(deps.storage, token_id.to_string(), &sale_info)?;
    Ok(Response::new()
        .add_attribute("action", "edit_sale")
        .add_attribute("token_id", token_id.to_string()))
}


pub fn execute_cancel_propose(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32,
) -> Result<Response, crate::ContractError> {

    let mut sale_info = SALE.load(deps.storage, token_id.to_string())?;
    let list = sale_info.requests.clone();
    let mut new_list: Vec<Request> = vec![];
    let mut cancel_price = Uint128::zero();

    for i in 0.. list.len() {
        if list[i].address == info.sender.clone() {
            cancel_price = list[i].price;
            continue;
        }
        new_list.push(list[i].clone());
    }
    
    sale_info.requests = new_list;

    SALE.save(deps.storage, token_id.to_string(), &sale_info)?;

    let mut msgs: Vec<CosmosMsg> = vec![];
    msgs.push(util::transfer_token_message(sale_info.denom.clone(), cancel_price, info.sender.clone())?);

    Ok(Response::new()
        .add_attribute("action", "cancel_propose")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("address", info.sender.clone().to_string())
        .add_messages(msgs)
    )
}


pub fn execute_change_contract(
    deps: DepsMut,
    info: MessageInfo,
    cw721_address: Addr
) -> Result<Response, crate::ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(crate::ContractError::Unauthorized {});
    }
    config.cw721_address = Some(cw721_address.clone());
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "change_contract")
        .add_attribute("cw721_address", cw721_address.to_string())
        .add_submessages(vec![]))
}

pub fn execute_change_cw721_owner(
    deps: DepsMut,
    info: MessageInfo,
    owner: Addr
) -> Result<Response, crate::ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if info.sender != config.owner {
        return Err(crate::ContractError::Unauthorized {});
    }

    let change_msg = Cw721ExecuteMsg::<Extension>::ChangeMinter {
        new_minter: owner.clone().into()
    };

    let callback = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.cw721_address.clone().unwrap().to_string(),
        msg: to_binary(&change_msg)?,
        funds: vec![],
    });

    Ok(Response::new()
        .add_message(callback)
        .add_attribute("action", "change_cw721_owner")
        .add_attribute("owner", owner.to_string())
        .add_submessages(vec![]))
}


pub fn execute_update_unused_token_id(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32
) -> Result<Response, crate::ContractError> {
    util::check_owner(deps.storage, info.sender.clone())?;
    let mut config = CONFIG.load(deps.storage)?;
    config.unused_token_id = token_id;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "change_unused_token_id")
        .add_attribute("token_id", token_id.to_string())
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
