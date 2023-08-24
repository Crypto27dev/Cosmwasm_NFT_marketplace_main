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
    pub price: Uint128,
    pub denom: String,
    pub cw721_address: Addr
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
    SetToken {
        token_id: String
    },
    Buy {},
    Withdraw {
        index: u32
    },
    WithdrawId {
        token_id: String
    }
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetToken {
        index: u32
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub price: Uint128,
    pub total_count: u32,
    pub sold_index: u32,
    pub cw721_address: Addr,
    pub enabled: bool,
    pub denom: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}