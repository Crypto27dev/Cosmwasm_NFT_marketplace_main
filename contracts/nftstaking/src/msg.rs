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
    pub collection_address: Addr,
    pub cw20_address: Addr,
    pub daily_reward: Uint128,
    pub interval: u64,
    pub lock_time: u64
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
    UpdateConfig {
        cw20_address: Addr,
        daily_reward: Uint128,
        interval: u64,
        lock_time: u64
    },
    ReceiveNft(Cw721ReceiveMsg),
    Claim {
    },
    CreateUnstake {
    },
    FetchUnstake {
    },
    WithdrawId {
        token_id: String
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NftReceiveMsg {
    Stake {
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetStaking {
        address: Addr
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: Addr,
    pub collection_address: Addr,
    pub cw20_address: Addr,
    pub daily_reward: Uint128,
    pub interval: u64,
    pub lock_time: u64,
    pub enabled: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingInfo {
    pub address: Addr,
    pub token_ids: Vec<String>,
    pub claimed_amount: Uint128,
    pub unclaimed_amount: Uint128,
    pub claimed_timestamp: u64,
    pub create_unstake_timestamp: u64,
    pub last_timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakingListResponse {
    pub list: Vec<StakingInfo>
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}