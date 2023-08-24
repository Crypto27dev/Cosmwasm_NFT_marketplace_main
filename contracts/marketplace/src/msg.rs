use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg};
use cosmwasm_std::{Uint128, Addr};

use marble_collection::msg::{InstantiateMsg as CollectionInstantiateMsg, ExecuteMsg as CollectionExecuteMsg};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub collection_code_id: u64,
    pub cw721_base_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        new_owner: Addr
    },
    UpdateConstants {
        collection_code_id: u64,
        cw721_base_code_id: u64,
    },
    // AddCollection {
    //     collection_addr: Addr,
    //     cw721_addr: Addr
    // },
    RemoveCollection {
        id: u32
    },
    RemoveAllCollection {

    },
    AddCollection(CollectionInstantiateMsg),
    EditCollection(CollectionInfo),
    EditUri {
        id: u32,
        uri: String
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Collection {
        id: u32
    },
    ListCollections {
        start_after: Option<u32>,
        limit: Option<u32>
    },
    OwnedCollections {
        owner: Addr
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub max_collection_id: u32,
    pub collection_code_id: u64,
    pub cw721_base_code_id: u64

}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CollectionRecord {
    pub owner: Addr,
    pub collection_address: Addr,
    pub cw721_address: Addr,
    pub uri: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CollectionInfo {
    pub id: u32,
    pub owner: Addr,
    pub collection_address: Addr,
    pub cw721_address: Addr,
    pub uri: String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CollectionListResponse {
    pub list: Vec<CollectionInfo>
}
