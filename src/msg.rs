use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use anone_cw721::msg::{InstantiateMsg as An721InstantiateMsg, RoyaltyInfoResponse};

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
    CreateModel {
        model_id: String,
        model_uri: String
    },
    UpdatePerModelShoeLimit {
        per_address_limit: u32,
    },
    UpdateAdmin {
        new_admin: String,
    },
    UpdateCollectionInfo {
        description: Option<String>,
        external_link: Option<String>,
        image: Option<String>,
        royalty_info: Option<RoyaltyInfoResponse>,
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
