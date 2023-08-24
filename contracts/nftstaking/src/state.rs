use cw721_base::Extension;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw_utils::{Expiration, Scheduled};
use cw_storage_plus::{Map};
use crate::msg::{StakingInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub collection_address: Addr,
    pub cw20_address: Addr,
    pub daily_reward: Uint128,
    pub interval: u64,
    pub lock_time: u64,
    pub enabled: bool
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const STAKING_KEY: &str = "staking";
pub const STAKING: Map<Addr, StakingInfo> = Map::new(STAKING_KEY);

