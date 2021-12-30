// ======================================================================
// Imports
// ======================================================================

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, OwnedDeps, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, HashResponse, InstantiateMsg, OwnerResponse, QueryMsg, ScoreFromAddressResponse};
use crate::state::{State, STATE};
use std::collections::HashMap;

extern crate serde_derive;
extern crate serde;
extern crate serde_json;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:ethan-gnibus-smart-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ======================================================================
// Instantiate Block
// ======================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    
    // Use msg.first_address_score to make a JSON string that will
    // hold the Key, Value pairs we will use to represent
    // Addresses and corresponding scores
    let address = msg.first_address;
    let score = msg.first_address_score;
    let mut hash: HashMap<String, i32> = HashMap::new();
    hash.insert(address, score);
    let hash = serde_json::to_string(&hash).unwrap().to_string();

    // Initialize state.
    let state = State {
        hash: hash.clone(),
        owner: info.sender.clone(),
    };

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("hash", hash)
        .add_attribute("owner", info.sender)
    )
}

// ======================================================================
// Execute Block
// ======================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // ExecuteMsg::Increment {} => try_increment(deps),
        // ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::AddAddress { new_address, new_score } => try_add_address(deps, info, new_address, new_score),
        ExecuteMsg::Set { address, new_score } => try_set(deps, info, address, new_score),
    }
}

pub fn try_add_address(deps: DepsMut, _info: MessageInfo, new_address: String, new_score: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // Deserialize the state HashMap from the JSON String.
        let mut deserialized: HashMap<String, i32> = serde_json::from_str(&state.hash).unwrap();
        
        // Error if now_address is already in the HashMap.
        if deserialized.contains_key(&new_address) {
            
            return Err(ContractError::Unauthorized {});
        }

        // insert the key value pair to the HashMap.
        deserialized.insert(
            new_address,
            new_score,
        );

        // Update the JSON String with the updated Hashmap.
        state.hash = serde_json::to_string(&deserialized).unwrap().to_string();
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "add_address"))
}

pub fn try_set(deps: DepsMut, info: MessageInfo, address: String, new_score: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        // Error if someone other than the owner is trying to set.
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }

        // Deserialize the state HashMap from the JSON String.
        let mut deserialized: HashMap<String, i32> = serde_json::from_str(&state.hash).unwrap();

        // Error if the address is not in the HashMap.
        if !deserialized.contains_key(&address) {
            return Err(ContractError::Unauthorized {});
        }

        // Update the score at the given address.
        *deserialized.get_mut(&address).unwrap() = new_score;

        // Update the JSON String with the updated Hashmap.
        state.hash = serde_json::to_string(&deserialized).unwrap().to_string();
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "set"))
}

// ======================================================================
// Query Block
// ======================================================================

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),
        QueryMsg::GetHash {} => to_binary(&query_hash(deps)?),
        QueryMsg::GetScoreFromAddress { address } => to_binary(&query_score_from_address(deps, address)?),
    }
}

fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(OwnerResponse { owner: state.owner })
}

fn query_hash(deps: Deps) -> StdResult<HashResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(HashResponse { hash: state.hash })
}

fn query_score_from_address(deps: Deps,  address: String) -> StdResult<ScoreFromAddressResponse> {
    let state = STATE.load(deps.storage)?;
    let deserialized: HashMap<String, i32> = serde_json::from_str(&state.hash).unwrap();
    let mut option = deserialized.get(&address);
    let score: i32 = **option.get_or_insert(&(1 as i32));
    Ok(ScoreFromAddressResponse { score: score })
}

// ======================================================================
// Testing Block
// ======================================================================
#[cfg(test)]
mod tests {

    // Testing Imports
    use super::*;
    use cosmwasm_std::testing::{MockApi, mock_dependencies, mock_env, mock_info, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, from_binary};

    pub fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo, InstantiateMsg) {
        // setup code specific to your library's tests would go here
        let deps = mock_dependencies(&[]);
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = InstantiateMsg {
            first_address: "1".to_string(),
            first_address_score: 10 as i32
        };
        return (deps, info, msg);
    }

    // ===========================
    // VERBOSE REQUIREMENT TESTS
    // ===========================

    // - you should be able to instantiate the contract and set the owner
    #[test]
    fn instantiate_contract_and_set_owner() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    // - you should support a read query to get the owner of the smart contract
    #[test]
    fn support_a_read_query_to_get_the_owner_of_the_start_contract() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!("owner", value.owner);
    }

    // - you should store the score for different addresses in the smart contract state (ex. {address_1: 10, address_2: 20}) 
    #[test]
    fn store_the_score_for_different_addresses_in_the_smart_contract_state() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Call AddAddress
        let info = mock_info("owner", &coins(1000, "earth"));
        let new_address = "2".to_string();
        let new_score = 20 as i32;
        let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Make sure Address1's score is 10.
        let address = "1".to_string();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);

        // Make sure Address2's score is 20.
        let address = "2".to_string();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 20 as i32);
    }

    // Ensure one cannot add an address if it already exists.
    #[test]
    fn error_if_adding_to_existing_address() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Call AddAddress
        let info = mock_info("owner", &coins(1000, "earth"));
        let new_address = "1".to_string();
        let new_score = 20 as i32;
        let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    // - you should support an execute message where only the owner of the smart contract can set the score of an address
    #[test]
    fn set_by_owner() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // beneficiary can release it
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 21 as i32};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // it worked, let's query the state
        // Make sure Address1's score is 10.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 21 as i32);
    }

    #[test]
    fn set_by_anyone() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 21 as i32};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Make sure Address1's score is 10.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);
    }

    // - you should support a read query to get the score for a particular address
    #[test]
    fn read_query_to_get_the_score_of_particular_address() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);
    }

    // ===========================
    // UNIT TESTS
    // ===========================

    // Ensure one cannot set at an invalid address.
    #[test]
    fn set_by_owner_at_invalid_address() {
        let (mut deps, info, msg) = setup();

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // beneficiary can release it
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = ExecuteMsg::Set { address: "2".to_string(), new_score: 21 as i32};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must provide a valid address to set."),
        }
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
