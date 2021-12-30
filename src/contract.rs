#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, HashResponse, InstantiateMsg, OwnerResponse, QueryMsg};
use crate::state::{State, STATE};
use std::collections::HashMap;

extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ethan-gnibus-smart-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    //msg.hash;
    let state = State {
        count: msg.count,
        hash: msg.hash.clone(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("hash", msg.hash)
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::Set { address, new_score } => try_set(deps, info, address, new_score),
    }
}

pub fn try_set(deps: DepsMut, info: MessageInfo, address: String, new_score: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }

        let mut deserialized: HashMap<String, i32> = serde_json::from_str(&state.hash).unwrap();
        *deserialized.get_mut(&address).unwrap() = new_score;
        let serialized = serde_json::to_string(&deserialized).unwrap();
        
        state.hash = String::from(serialized);
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "set"))
}


pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}
pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetHash {} => to_binary(&query_hash(deps)?),
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

fn query_hash(deps: Deps) -> StdResult<HashResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(HashResponse { hash: state.hash })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    // ===========================
    // Unit TESTS
    // ===========================
 
    #[test]
    fn init_key_1_val_10() {
        let mut deps = mock_dependencies(&[]);
        let mut hash: HashMap<String, i32> = HashMap::new();
        hash.insert(String::from("1"), 10);

        let serialized = serde_json::to_string(&hash).unwrap();

        let hash: String = String::from(serialized);

        let msg = InstantiateMsg { count: 0, hash: hash};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetHash {}).unwrap();
        let value: HashResponse = from_binary(&res).unwrap();
        let hash = value.hash;
        let deserialized: HashMap<String, i32> = serde_json::from_str(&hash).unwrap();
        let mut option = deserialized.get(&String::from("1"));
        let score: &mut &i32 = option.get_or_insert(&(1 as i32));
        let expected: i32 = 10 as i32;
        assert_eq!(**score, expected);
    }

    #[test]
    fn set_by_creator() {
        let mut deps = mock_dependencies(&[]);
        let mut hash: HashMap<String, i32> = HashMap::new();
        hash.insert(String::from("1"), 10);
        let serialized = serde_json::to_string(&hash).unwrap();
        let hash: String = String::from(serialized);
        let msg = InstantiateMsg { count: 0, hash: hash};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // beneficiary can release it
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::Set { address: String::from("1"), new_score: 21 as i32};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetHash {}).unwrap();
        let value: HashResponse = from_binary(&res).unwrap();
        let hash = value.hash;
        let deserialized: HashMap<String, i32> = serde_json::from_str(&hash).unwrap();
        let mut option = deserialized.get(&String::from("1"));
        let score: &mut &i32 = option.get_or_insert(&(1 as i32));
        let expected: i32 = 21 as i32;
        assert_eq!(**score, expected);
    }

    #[test]
    fn set_by_anyone() {
        let mut deps = mock_dependencies(&[]);
        let mut hash: HashMap<String, i32> = HashMap::new();
        hash.insert(String::from("1"), 10);
        let serialized = serde_json::to_string(&hash).unwrap();
        let hash: String = String::from(serialized);
        let msg = InstantiateMsg { count: 0, hash: hash};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Set { address: String::from("1"), new_score: 21 as i32};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }
    }


    // #[test]
    // fn proper_initialization_with_0() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(0, value.count);
    // }

    // #[test]
    // fn proper_initialization_with_17() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }

    // #[test]
    // fn proper_initialization_with_negative() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: -1 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(-1, value.count);
    // }

    // #[test]
    // fn double_initialization() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(0, value.count);

    //     // Instantiating the second time
    //     let msg = InstantiateMsg { count: 50 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(50, value.count);
    // }


    // #[test]
    // fn increment_by_anyone() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn increment_by_creator() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn increment_twice_one_incrementer() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // Increment again with the same info
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 2
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(19, value.count);
    // }

    // #[test]
    // fn increment_twice_multiple_incrementers() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // increment again with different info
    //     let info = mock_info("someone_else", &coins(4, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(19, value.count);
    // }

    // #[test]
    // fn increment_51_times_by_one_incrementer() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    //     // increment 51 times
    //     for _n in 0..51 {
    //         // beneficiary can release it
    //         let info = mock_info("someone", &coins(2, "token"));
    //         let msg = ExecuteMsg::Increment {};
    //         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     }

    //     // should increase counter by 51
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(51, value.count);
    // }

    // #[test]
    // fn increment_51_times_by_multiple_incrementers() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    //     // increment 51 times
    //     for n in 0..51 {
    //         // beneficiary can release it
    //         let info = mock_info("incrementer{n}", &coins(n, "token"));
    //         let msg = ExecuteMsg::Increment {};
    //         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     }

    //     // should increase counter by 51
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(51, value.count);
    // }

    // #[test]
    // fn reset_by_creator() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }

    // #[test]
    // fn reset_by_anyone() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }
    // }

    // #[test]
    // fn reset_twice() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);

    //     // testing non creator trying to reset again
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 10 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // testing creator resetting to 10
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 10 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 10
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(10, value.count);
    // }

    // #[test]
    // fn reset_42_times() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     for n in 0..42 {
    //         // beneficiary can release it
    //         let unauth_info = mock_info("anyone{n}", &coins(2, "token"));
    //         let msg = ExecuteMsg::Reset { count: n };
    //         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //         match res {
    //             Err(ContractError::Unauthorized {}) => {}
    //             _ => panic!("Must return unauthorized error"),
    //         }

    //         // only the original creator can reset the counter
    //         let auth_info = mock_info("creator", &coins(2, "token"));
    //         let msg = ExecuteMsg::Reset { count: n };
    //         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //         // should now be n
    //         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //         let value: CountResponse = from_binary(&res).unwrap();
    //         assert_eq!(n, value.count);
    //     }
    // }

    // // ===========================
    // // INTEGRATION TESTS
    // // ===========================

    // #[test]
    // fn increment_then_reset() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 8 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    //     // Increment again with the same info
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(9, value.count);

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 3 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 3 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 3
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(3, value.count);
    // }

    // #[test]
    // fn increment_then_reset_twice() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        
    //     // Increment again with the same info
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 2
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
        
    //     // Increment again with the same info
    //     let info = mock_info("someone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Increment {};
    //     let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(6, value.count);

    //     // testing non creator trying to reset again
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 10 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // testing creator resetting to 10
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 10 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 10
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(10, value.count);
    // }

    // #[test]
    // fn editing_multiple_addresses_test() {
    //     let mut deps1 = mock_dependencies(&coins(1, "earth"));
    //     let mut deps2 = mock_dependencies(&coins(2, "earth"));

    //     // // Instantiating address_1 with a score of 10
    //     let msg1 = InstantiateMsg { count: 10 };
    //     let info1 = mock_info("creator", &coins(1, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res1 = instantiate(deps1.as_mut(), mock_env(), info1, msg1).unwrap();
    //     assert_eq!(0, res1.messages.len());

    //     // Instantiating address_2 with a score of 20
    //     let msg2 = InstantiateMsg { count: 20 };
    //     let info2 = mock_info("creator", &coins(2, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res2 = instantiate(deps2.as_mut(), mock_env(), info2, msg2).unwrap();
    //     assert_eq!(0, res2.messages.len());

    //     // it worked, let's query the state
    //     let res1 = query(deps1.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value1: CountResponse = from_binary(&res1).unwrap();
    //     assert_eq!(10, value1.count);

    //     // ensure address_2's count (score) is 20
    //     let res2 = query(deps2.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value2: CountResponse = from_binary(&res2).unwrap();
    //     assert_eq!(20, value2.count);

    //     // reset the count (score) at address 2 to be 15
    //     let auth_info = mock_info("creator", &coins(2, "earth"));
    //     let msg2 = ExecuteMsg::Reset { count: 15 };
    //     let _res = execute(deps2.as_mut(), mock_env(), auth_info, msg2).unwrap();

    //     // address 1 shoud stay at 10
    //     let res1 = query(deps1.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value1: CountResponse = from_binary(&res1).unwrap();
    //     assert_eq!(10, value1.count);

    //     // address 2 should now be 15
    //     let res2 = query(deps2.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value2: CountResponse = from_binary(&res2).unwrap();
    //     assert_eq!(15, value2.count);

    //     // increment the count (score) at address 1 so that it's 11
    //     let info1 = mock_info("creator", &coins(2, "token"));
    //     let msg1 = ExecuteMsg::Increment {};
    //     let _res = execute(deps1.as_mut(), mock_env(), info1, msg1).unwrap();

    //     // address 1 shoud now be 11
    //     let res1 = query(deps1.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value1: CountResponse = from_binary(&res1).unwrap();
    //     assert_eq!(11, value1.count);

    //     // address 2 should stay at 15
    //     let res2 = query(deps2.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value2: CountResponse = from_binary(&res2).unwrap();
    //     assert_eq!(15, value2.count);
    // }

    // #[test]
    // fn ensure_address_owners_can_not_reset_others_addresses() {
    //     let mut deps1 = mock_dependencies(&coins(1, "earth"));
    //     let mut deps2 = mock_dependencies(&coins(2, "earth"));

    //     // // Instantiating address_1 with a score of 10
    //     let msg1 = InstantiateMsg { count: 10 };
    //     let info1 = mock_info("Alice", &coins(1, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res1 = instantiate(deps1.as_mut(), mock_env(), info1, msg1).unwrap();
    //     assert_eq!(0, res1.messages.len());

    //     // Instantiating address_2 with a score of 20
    //     let msg2 = InstantiateMsg { count: 20 };
    //     let info2 = mock_info("Bob", &coins(2, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res2 = instantiate(deps2.as_mut(), mock_env(), info2, msg2).unwrap();
    //     assert_eq!(0, res2.messages.len());

    //     // it worked, let's query the state
    //     let res1 = query(deps1.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value1: CountResponse = from_binary(&res1).unwrap();
    //     assert_eq!(10, value1.count);

    //     // ensure address_2's count (score) is 20
    //     let res2 = query(deps2.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value2: CountResponse = from_binary(&res2).unwrap();
    //     assert_eq!(20, value2.count);

    //     // ensure Alice can't reset Bob's
    //     let info = mock_info("Bob", &coins(2, "earth"));
    //     let msg = ExecuteMsg::Reset { count: 100 };
    //     let res = execute(deps1.as_mut(), mock_env(), info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // ensure Bob can't reset Alice's
    //     let info = mock_info("Alice", &coins(1, "earth"));
    //     let msg = ExecuteMsg::Reset { count: 100 };
    //     let res = execute(deps2.as_mut(), mock_env(), info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }
    // }

    // // ===========================
    // // VERBOSE REQUIREMENT TESTS
    // // ===========================
    // // - you should be able to instantiate the contract and set the owner 
    // #[test]
    // fn instantiate_the_contract_and_set_the_owner() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("owner", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(0, value.count);
    // }

    // // - you should support a read query to get the owner of the smart contract 
    // #[test]
    // fn support_a_read_query_to_get_the_owner_of_the_start_contract() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 0 };
    //     let info = mock_info("owner", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     //assert_eq!("owner", res.attributes[1].key);
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
    //     let value: OwnerResponse = from_binary(&res).unwrap();
    //     assert_eq!("owner", value.owner);
    // }

    // // - you should store the score for different addresses in the smart contract state (ex. {address_1: 10, address_2: 20})
    // #[test]
    // fn store_the_score_for_different_addresses_in_the_smart_contract_state() {
    //     let mut deps1 = mock_dependencies(&coins(1, "earth"));
    //     let mut deps2 = mock_dependencies(&coins(2, "earth"));

    //     // // Instantiating address_1 with a score of 10
    //     let msg1 = InstantiateMsg { count: 10 };
    //     let info1 = mock_info("creator", &coins(1, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res1 = instantiate(deps1.as_mut(), mock_env(), info1, msg1).unwrap();
    //     assert_eq!(0, res1.messages.len());

    //     // Instantiating address_2 with a score of 20
    //     let msg2 = InstantiateMsg { count: 20 };
    //     let info2 = mock_info("creator", &coins(2, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res2 = instantiate(deps2.as_mut(), mock_env(), info2, msg2).unwrap();
    //     assert_eq!(0, res2.messages.len());

    //     // it worked, let's query the state
    //     let res1 = query(deps1.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value1: CountResponse = from_binary(&res1).unwrap();
    //     assert_eq!(10, value1.count);

    //     // ensure address_2's count (score) is 20
    //     let res2 = query(deps2.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value2: CountResponse = from_binary(&res2).unwrap();
    //     assert_eq!(20, value2.count);
    // }

    // // - you should support an execute message where only the owner of the smart contract can set the score of an address 
    // #[test]
    // fn execute_message_where_only_the_contract_owner_can_set_the_score_of_an_address() {
    //     let mut deps = mock_dependencies(&coins(2, "token"));

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(2, "token"));
    //     let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    //     // beneficiary can release it
    //     let unauth_info = mock_info("anyone", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
    //     match res {
    //         Err(ContractError::Unauthorized {}) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_info = mock_info("creator", &coins(2, "token"));
    //     let msg = ExecuteMsg::Reset { count: 5 };
    //     let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

    //     // should now be 5
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }

    // // - you should support an execute message where only the owner of the smart contract can set the score of an address
    // #[test]
    // fn read_query_to_get_the_score_of_particular_address() {
    //     let mut deps = mock_dependencies(&[]);

    //     let msg = InstantiateMsg { count: 10 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(10, value.count);
    // }
}
