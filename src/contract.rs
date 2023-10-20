use crate::ContractError::Unauthorized;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};
use std::collections::BTreeMap;
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{Deposit, ExecuteMsg, GetJobIdResponse, InstantiateMsg, PalomaMsg, QueryMsg};
use crate::state::{State, STATE};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        job_id: msg.job_id.clone(),
        owner: info.sender.clone(),
    };
    STATE.save(deps.storage, &state)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("job_id", msg.job_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::PutSwap { deposit } => swap(deps, info, deposit),
        ExecuteMsg::SetPaloma {} => set_paloma(deps, info),
        ExecuteMsg::UpdateCompass { new_compass } => update_compass(deps, info, new_compass),
    }
}

fn swap(
    deps: DepsMut,
    info: MessageInfo,
    deposits: Deposit,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "swap".to_string(),
            vec![Function {
                name: "swap".to_string(),
                inputs: vec![
                    Param {
                        name: "receiver".to_string(),
                        kind: ParamType::Address,
                        internal_type: None,
                    },
                    Param {
                        name: "amount".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                    Param {
                        name: "expected".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                    Param {
                        name: "deposit_id".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                ],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    let tokens = vec![
        Token::Address(Address::from_str(deposits.receiver.as_str()).unwrap()),
        Token::Uint(Uint::from_big_endian(&deposits.amount.to_be_bytes())),
        Token::Uint(Uint::from_big_endian(&deposits.expected.to_be_bytes())),
        Token::Uint(Uint::from_big_endian(&deposits.deposit_id.to_be_bytes())),
    ];
    let state = STATE.load(deps.storage)?;
    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("swap")
                    .unwrap()
                    .encode_input(tokens.as_slice())
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "swap"))
}

fn set_paloma(deps: DepsMut, info: MessageInfo) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "set_paloma".to_string(),
            vec![Function {
                name: "set_paloma".to_string(),
                inputs: vec![],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };
    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("set_paloma")
                    .unwrap()
                    .encode_input(&[])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "set_paloma"))
}

fn update_compass(
    deps: DepsMut,
    info: MessageInfo,
    new_compass: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    let new_compass_address: Address = Address::from_str(new_compass.as_str()).unwrap();
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "update_compass".to_string(),
            vec![Function {
                name: "update_compass".to_string(),
                inputs: vec![Param {
                    name: "new_compass".to_string(),
                    kind: ParamType::Address,
                    internal_type: None,
                }],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("update_compass")
                    .unwrap()
                    .encode_input(&[Token::Address(new_compass_address)])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "update_compass"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetJobId {} => to_binary(&get_job_id(deps)?),
    }
}

pub fn get_job_id(deps: Deps) -> StdResult<GetJobIdResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetJobIdResponse {
        job_id: state.job_id,
    })
}
