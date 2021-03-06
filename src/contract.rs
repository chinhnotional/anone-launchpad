use anone_cw721::msg::{
    CreateShoeModelMsg, ExecuteMsg as An721ExecuteMsg, InstantiateMsg as An721InstantiateMsg,
    MintMsg as An721MintMsg, RoyaltyInfoResponse,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Addr, BankMsg, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Order,
    Reply, ReplyOn, Response, StdError, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use cw_utils::parse_reply_instantiate_data;

use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MintCountResponse, MintableNumTokensResponse,
    QueryMsg,
};
use crate::state::{
    Config, AN721_ADDRESS, CONFIG, MINTABLE_NUM_TOKENS, MINTABLE_TOKEN_IDS, MINTER_ADDRS,
};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:anone-launchpad";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_AN721_REPLY_ID: u64 = 1;

// governance parameters
pub const NATIVE_DENOM: &str = "uan1";
const MAX_TOKEN_LIMIT: u32 = 100000;
const MAX_PER_ADDRESS_LIMIT: u32 = 50;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Check the number of tokens is more than zero and less than the max limit
    if msg.num_tokens == 0 || msg.num_tokens > MAX_TOKEN_LIMIT {
        return Err(ContractError::InvalidNumTokens {
            min: 1,
            max: MAX_TOKEN_LIMIT,
        });
    }

    // Check per address limit is valid
    if msg.per_address_limit == 0 || msg.per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT,
            min: 1,
            got: msg.per_address_limit,
        });
    }

    let config = Config {
        admin: info.sender.clone(),
        num_tokens: msg.num_tokens,
        an721_code_id: msg.an721_code_id,
        per_address_limit: msg.per_address_limit,
    };
    CONFIG.save(deps.storage, &config)?;
    MINTABLE_NUM_TOKENS.save(deps.storage, &msg.num_tokens)?;

    // Save mintable token ids map
    for token_id in 1..=msg.num_tokens {
        MINTABLE_TOKEN_IDS.save(deps.storage, token_id, &true)?;
    }

    // Submessage to instantiate anone-cw721 contract
    let sub_msgs: Vec<SubMsg> = vec![SubMsg {
        msg: WasmMsg::Instantiate {
            code_id: msg.an721_code_id,
            msg: to_binary(&An721InstantiateMsg {
                name: msg.an721_instantiate_msg.name,
                symbol: msg.an721_instantiate_msg.symbol,
                minter: env.contract.address.to_string(),
                collection_info: msg.an721_instantiate_msg.collection_info,
            })?,
            funds: info.funds,
            admin: Some(info.sender.to_string()),
            label: String::from("Fixed price minter"),
        }
        .into(),
        id: INSTANTIATE_AN721_REPLY_ID,
        gas_limit: None,
        reply_on: ReplyOn::Success,
    }];

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessages(sub_msgs))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint { model_id, size } => execute_mint_sender(deps, info, model_id, size),
        ExecuteMsg::MintTo {
            recipient,
            model_id,
            size,
        } => execute_mint_to(deps, info, recipient, model_id, size),
        ExecuteMsg::MintFor {
            token_id,
            recipient,
            model_id,
            size,
        } => execute_mint_for(deps, info, token_id, recipient, model_id, size),
        ExecuteMsg::CreateModel {
            model_id,
            model_uri,
        } => execute_create_model(deps, info, model_id, model_uri),
        ExecuteMsg::UpdatePerModelShoeLimit { per_address_limit } => {
            execute_update_per_address_limit(deps, env, info, per_address_limit)
        }
        ExecuteMsg::UpdateCollectionInfo {
            description,
            external_link,
            image,
            royalty_info,
        } => execute_update_collection_info(
            deps,
            info,
            description,
            external_link,
            image,
            royalty_info,
        ),
        ExecuteMsg::UpdateAdmin { new_admin } => execute_update_admin(deps, env, info, new_admin),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
    }
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.admin != info.sender {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    };

    // query balance from the contract
    let balance = deps
        .querier
        .query_balance(env.contract.address, NATIVE_DENOM)?;
    if balance.amount.is_zero() {
        return Err(ContractError::ZeroBalance {});
    }

    // send contract balance to creator
    let send_msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![balance],
    });

    Ok(Response::default()
        .add_attribute("action", "withdraw")
        .add_message(send_msg))
}

pub fn execute_create_model(
    deps: DepsMut,
    info: MessageInfo,
    model_id: String,
    model_uri: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let action = "create_model";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let an721_address = AN721_ADDRESS.load(deps.storage)?;
    // Create create_model msgs
    let create_model_msg = An721ExecuteMsg::CreateShoeModel(CreateShoeModelMsg::<Empty> {
        model_id: model_id.clone(),
        owner: info.sender.clone().to_string(),
        model_uri: model_uri,
        extension: Empty {},
    });
    let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: an721_address.to_string(),
        msg: to_binary(&create_model_msg)?,
        funds: vec![],
    });
    Ok(Response::default()
        .add_attribute("action", action)
        .add_attribute("sender", info.sender)
        .add_attribute("model_id", model_id)
        .add_attribute("owner", config.admin.to_string())
        .add_message(msg))
}

pub fn execute_mint_sender(
    deps: DepsMut,
    info: MessageInfo,
    model_id: String,
    size: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let action = "mint_sender";

    // Check if already minted max per address limit
    let mint_count = mint_count(deps.as_ref(), &info)?;
    if mint_count >= config.per_address_limit {
        return Err(ContractError::MaxPerAddressLimitExceeded {});
    }

    _execute_mint(deps, info, action, None, None, model_id, size)
}

pub fn execute_mint_to(
    deps: DepsMut,
    info: MessageInfo,
    recipient: String,
    model_id: String,
    size: String,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    let config = CONFIG.load(deps.storage)?;
    let action = "mint_to";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    _execute_mint(deps, info, action, Some(recipient), None, model_id, size)
}

pub fn execute_mint_for(
    deps: DepsMut,
    info: MessageInfo,
    token_id: u32,
    recipient: String,
    model_id: String,
    size: String,
) -> Result<Response, ContractError> {
    let recipient = deps.api.addr_validate(&recipient)?;
    let config = CONFIG.load(deps.storage)?;
    let action = "mint_for";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    _execute_mint(
        deps,
        info,
        action,
        Some(recipient),
        Some(token_id),
        model_id,
        size,
    )
}

// Generalize checks and mint message creation
// mint -> _execute_mint(recipient: None, token_id: None)
// mint_to(recipient: "friend") -> _execute_mint(Some(recipient), token_id: None)
// mint_for(recipient: "friend2", token_id: 420) -> _execute_mint(recipient, token_id)
fn _execute_mint(
    deps: DepsMut,
    info: MessageInfo,
    action: &str,
    recipient: Option<Addr>,
    token_id: Option<u32>,
    model_id: String,
    size: String,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let an721_address = AN721_ADDRESS.load(deps.storage)?;

    let recipient_addr = match recipient {
        Some(some_recipient) => some_recipient,
        None => info.sender.clone(),
    };

    let mut msgs: Vec<CosmosMsg<Empty>> = vec![];

    let mintable_token_id = match token_id {
        Some(token_id) => {
            if token_id == 0 || token_id > config.num_tokens {
                return Err(ContractError::InvalidTokenId {});
            }
            // If token_id not on mintable map, throw err
            if !MINTABLE_TOKEN_IDS.has(deps.storage, token_id) {
                return Err(ContractError::TokenIdAlreadySold { token_id });
            }
            token_id
        }
        None => {
            let mintable_tokens_result: StdResult<Vec<u32>> = MINTABLE_TOKEN_IDS
                .keys(deps.storage, None, None, Order::Ascending)
                .take(1)
                .collect();
            let mintable_tokens = mintable_tokens_result?;
            if mintable_tokens.is_empty() {
                return Err(ContractError::SoldOut {});
            }
            mintable_tokens[0]
        }
    };

    // Create mint msgs
    let mint_msg = An721ExecuteMsg::Mint(An721MintMsg::<Empty> {
        token_id: mintable_token_id.to_string(),
        owner: recipient_addr.to_string(),
        model_id: model_id,
        size: size,
        extension: Empty {},
    });
    let msg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: an721_address.to_string(),
        msg: to_binary(&mint_msg)?,
        funds: vec![],
    });
    msgs.append(&mut vec![msg]);

    // Remove mintable token id from map
    MINTABLE_TOKEN_IDS.remove(deps.storage, mintable_token_id);
    let mintable_num_tokens = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    // Decrement mintable num tokens
    MINTABLE_NUM_TOKENS.save(deps.storage, &(mintable_num_tokens - 1))?;
    // Save the new mint count for the sender's address
    let new_mint_count = mint_count(deps.as_ref(), &info)? + 1;
    MINTER_ADDRS.save(deps.storage, info.clone().sender, &new_mint_count)?;

    Ok(Response::default()
        .add_attribute("action", action)
        .add_attribute("sender", info.sender)
        .add_attribute("recipient", recipient_addr)
        .add_attribute("token_id", mintable_token_id.to_string())
        .add_messages(msgs))
}

pub fn execute_update_per_address_limit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    per_address_limit: u32,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }
    if per_address_limit == 0 || per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT,
            min: 1,
            got: per_address_limit,
        });
    }
    config.per_address_limit = per_address_limit;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_per_address_limit")
        .add_attribute("sender", info.sender)
        .add_attribute("limit", per_address_limit.to_string()))
}

pub fn execute_update_collection_info(
    deps: DepsMut,
    info: MessageInfo,
    description: Option<String>,
    external_link: Option<String>,
    image: Option<String>,
    royalty_info: Option<RoyaltyInfoResponse>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let action = "update_collection_info";

    // Check only admin
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let an721_address = AN721_ADDRESS.load(deps.storage)?;
    // Create create_model msgs
    let update_collection_info_msg: An721ExecuteMsg<Empty> =
        An721ExecuteMsg::ModifyCollectionInfo {
            description: description.clone(),
            external_link: external_link.clone(),
            image: image.clone(),
            royalty_info: royalty_info.clone(),
        };
    let msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: an721_address.to_string(),
        msg: to_binary(&update_collection_info_msg)?,
        funds: vec![],
    });
    Ok(Response::default()
        .add_attribute("action", action)
        .add_attribute("sender", info.sender)
        .add_message(msg))
}

pub fn execute_update_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    if info.sender != config.admin {
        return Err(ContractError::Unauthorized(
            "Sender is not an admin".to_owned(),
        ));
    }

    let admin = deps.api.addr_validate(&new_admin)?;
    config.admin = admin;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_admin")
        .add_attribute("sender", info.sender)
        .add_attribute("new_admin", new_admin.to_string()))
}

fn mint_count(deps: Deps, info: &MessageInfo) -> Result<u32, StdError> {
    let mint_count = (MINTER_ADDRS
        .key(info.sender.clone())
        .may_load(deps.storage)?)
    .unwrap_or(0);
    Ok(mint_count)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::MintableNumTokens {} => to_binary(&query_mintable_num_tokens(deps)?),
        QueryMsg::MintCount { address } => to_binary(&query_mint_count(deps, address)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let an721_address = AN721_ADDRESS.load(deps.storage)?;

    Ok(ConfigResponse {
        admin: config.admin.to_string(),
        an721_address: an721_address.to_string(),
        an721_code_id: config.an721_code_id,
        num_tokens: config.num_tokens,
        per_address_limit: config.per_address_limit,
    })
}

fn query_mint_count(deps: Deps, address: String) -> StdResult<MintCountResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let mint_count = (MINTER_ADDRS.key(addr.clone()).may_load(deps.storage)?).unwrap_or(0);
    Ok(MintCountResponse {
        address: addr.to_string(),
        count: mint_count,
    })
}

fn query_mintable_num_tokens(deps: Deps) -> StdResult<MintableNumTokensResponse> {
    let count = MINTABLE_NUM_TOKENS.load(deps.storage)?;
    Ok(MintableNumTokensResponse { count })
}

// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_AN721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            AN721_ADDRESS.save(deps.storage, &Addr::unchecked(res.contract_address))?;
            Ok(Response::default().add_attribute("action", "instantiate_an721_reply"))
        }
        Err(_) => Err(ContractError::InstantiateAn721Error {}),
    }
}
