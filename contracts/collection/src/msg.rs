use cosmwasm_std::{Addr, Uint128};
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use cw721_base::Extension;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_utils::{Expiration, Scheduled};
use cw20::Denom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub max_tokens: u32,
    pub name: String,
    pub symbol: String,
    pub token_code_id: u64,
    pub maximum_royalty_fee: u32,
    pub royalties: Vec<Royalty>,
    pub uri: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr,
    },
    UpdateEnabled {
        enabled: bool
    },
    UpdateRoyalties {
        maximum_royalty_fee: u32,
        royalties: Vec<Royalty>
    },
    Mint {uri: String, extension: Extension},
    // Edit {token_id: u32, uri: String, extension: Extension},
    BatchMint {
        uri: Vec<String>, 
        extension:Vec<Extension>,
        owner: Vec<String>
    },
    Propose {
        token_id: u32,
        denom: String
    },
    Receive(Cw20ReceiveMsg),
    ReceiveNft(Cw721ReceiveMsg),
    AcceptSale {
        token_id: u32
    },
    CancelSale {
        token_id: u32,
    },
    ChangeContract {
        cw721_address: Addr
    },
    ChangeCw721Owner {
        owner: Addr
    },
    UpdateUnusedTokenId {
        token_id: u32
    },
    EditSale {
        token_id: u32,
        sale_type: SaleType,
        duration_type: DurationType,
        initial_price: Uint128,
        reserve_price: Uint128,
        denom: Denom
    },
    CancelPropose {
        token_id: u32
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Propose {
        token_id: u32
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NftReceiveMsg {
    StartSale {
        sale_type: SaleType,
        duration_type: DurationType,
        initial_price: Uint128,
        reserve_price: Uint128,
        denom: Denom
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetSale {
        token_id: u32,
    },
    GetSales {
        start_after: Option<u32>,
        limit: Option<u32>
    },
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub cw721_address: Option<Addr>,
    pub max_tokens: u32,
    pub name: String,
    pub symbol: String,
    pub unused_token_id: u32,
    pub maximum_royalty_fee: u32,
    pub royalties: Vec<Royalty>,
    pub uri: String,
    pub enabled: bool
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Request {
    pub address: Addr,
    pub price: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum SaleType {
    Fixed,
    Auction
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TimeDuration {
    pub start: u64,
    pub end: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum DurationType {
    Fixed,
    // Time(TimeDuration),
    Time(u64, u64),
    Bid(u32)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalty {
    pub address: Addr,
    pub rate: u32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SaleInfo {
    pub token_id: u32,
    pub provider: Addr,
    pub sale_type: SaleType,
    pub duration_type: DurationType,
    pub initial_price: Uint128,
    pub reserve_price: Uint128,
    pub requests: Vec<Request>,
    pub denom: Denom,
    pub can_accept: bool 
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SalesResponse {
    pub list: Vec<SaleInfo>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}