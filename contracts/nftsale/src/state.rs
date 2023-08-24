use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Item;
use cw_storage_plus::{Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub price: Uint128,
    pub total_count: u32,
    pub sold_index: u32,
    pub cw721_address: Addr,
    pub enabled: bool,
    pub denom: String
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const TOKENS_KEY: &str = "tokens";
pub const TOKENS: Map<u32, String> = Map::new(TOKENS_KEY);

