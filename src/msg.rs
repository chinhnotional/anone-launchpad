use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anone_cw721::msg::InstantiateMsg as An721InstantiateMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub num_tokens: u32,
    pub an721_code_id: u64,
    pub an721_instantiate_msg: An721InstantiateMsg,
    pub per_address_limit: u32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint {
        model_id: String,
        size: String,
    },
    UpdatePerModelShoeLimit {
        per_address_limit: u32,
    },
    UpdateAdmin {
        new_admin: String,
    },
    MintTo {
        recipient: String,
        model_id: String,
        size: String,
    },
    MintFor {
        token_id: u32,
        recipient: String,
        model_id: String,
        size: String,
    },
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MintableNumTokens {},
    MintCount { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub num_tokens: u32,
    pub per_address_limit: u32,
    pub an721_address: String,
    pub an721_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintableNumTokensResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}
