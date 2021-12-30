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

// version info for migration info.
const CONTRACT_NAME: &str = "crates.io:ethan-gnibus-smart-contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ======================================================================
// Instantiate Block
// ======================================================================

/// Instantiate a smart contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    
    // Use msg.first_address_score to make a JSON string that will
    // hold the Key, Value pairs we will use to represent
    // addresses and corresponding scores.
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

    // Save state.
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    // Return response.
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("hash", hash)
        .add_attribute("owner", info.sender)
    )
}

// ======================================================================
// Execute Block
// ======================================================================

/// Execute a command that will change a smart contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // When the msg matches AddAddress, call try_add_address.
        ExecuteMsg::AddAddress { new_address, new_score } => try_add_address(deps, info, new_address, new_score),

        // When the msg matches Set, call try_set.
        ExecuteMsg::Set { address, new_score } => try_set(deps, info, address, new_score),
    }
}

/// Adds the (address, score) pair to the smart contract iff the address if valid.
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

    // Return response.
    Ok(Response::new().add_attribute("method", "add_address"))
}

/// Updates the score at the given address iff the address is valid.
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

    // Return response.
    Ok(Response::new().add_attribute("method", "set"))
}

// ======================================================================
// Query Block
// ======================================================================

/// Calls a query that will not change the smart contract's contents.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        // When the msg matches GetOwner, call query_owner.
        QueryMsg::GetOwner {} => to_binary(&query_owner(deps)?),

        // When the msg matches GetHash, call query_hash.
        QueryMsg::GetHash {} => to_binary(&query_hash(deps)?),

        // When the msg matches GetScoreFromAddress,
        // call query_query_score_from_addressowner.
        QueryMsg::GetScoreFromAddress { address } => to_binary(&query_score_from_address(deps, address)?),
    }
}

/// Return the owner of the provided smart contract.
fn query_owner(deps: Deps) -> StdResult<OwnerResponse> {
    // Load the state.
    let state = STATE.load(deps.storage)?;

    // Return a response containing the owner's name.
    Ok(OwnerResponse { owner: state.owner })
}

/// Return the HashMap of addresses and cooresponding
/// scores converted to a JSON String that cooresponds
/// to the provided smart contract.
fn query_hash(deps: Deps) -> StdResult<HashResponse> {
    // Load the state.
    let state = STATE.load(deps.storage)?;

    // Return a response containing the state HashMap as a JSON String.
    Ok(HashResponse { hash: state.hash })
}

/// Return the acore the corresponds to the given address and smart contract.
fn query_score_from_address(deps: Deps,  address: String) -> StdResult<ScoreFromAddressResponse> {
    // Load the state.
    let state = STATE.load(deps.storage)?;

    // Deserialize the state HashMap.
    let deserialized: HashMap<String, i32> = serde_json::from_str(&state.hash).unwrap();

    // Get score at provided address.
    let mut option = deserialized.get(&address);
    let score: i32 = **option.get_or_insert(&(1 as i32));

    // Return a response containing the score at the provided address.
    Ok(ScoreFromAddressResponse { score: score })
}

// ======================================================================
// Testing Block
// ======================================================================

#[cfg(test)]
mod tests {
    // Imports for testing purposes.
    use super::*;
    use cosmwasm_std::testing::{MockApi, mock_dependencies, mock_env, mock_info, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, from_binary};

    /// A "DO BEFORE EACH" testing utility function.
    /// Returns the parameters necessary to instantiate a smart contract.
    pub fn setup() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo, InstantiateMsg) {
        // Mock out dependencies.
        let deps = mock_dependencies(&[]);

        // Mock out account info.
        let info = mock_info("owner", &coins(1000, "earth"));

        // Create a message that could be used to instantiate a smart contract.
        let msg = InstantiateMsg {
            first_address: "1".to_string(),
            first_address_score: 10 as i32
        };

        // Return all three to be used in test cases.
        return (deps, info, msg);
    }

    // ===========================
    // VERBOSE REQUIREMENT TESTS
    // ===========================

    /// Test instantiating the contract and setting the owner.
    #[test]
    fn instantiate_contract_and_set_owner() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Ensure instantiation was successful.
        assert_eq!(0, res.messages.len());
    }

    /// Test making a read query to get the owner of the smart contract.
    #[test]
    fn support_a_read_query_to_get_the_owner_of_the_start_contract() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract. 
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Ensure the read query returns the owner of the smart contract. 
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetOwner {}).unwrap();
        let value: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!("owner", value.owner);
    }

    /// Testing storing scores for different addresses in the smart contract state.
    /// (ex. {address_1: 10, address_2: 20})
    #[test]
    fn store_the_score_for_different_addresses_in_the_smart_contract_state() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Call AddAddress.
        let info = mock_info("owner", &coins(1000, "earth"));
        let new_address = "2".to_string();
        let new_score = 20 as i32;
        let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Ensure Address1's score is 10.
        let address = "1".to_string();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);

        // Ensure Address2's score is 20.
        let address = "2".to_string();
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 20 as i32);
    }

    /// Testing support for an execute message where only the owner
    /// of the smart contract can set the score of an address.
    /// Case: Owner.
    #[test]
    fn set_by_owner() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Execute Set as owner.
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 21 as i32};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Make sure Address1's score is 21.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 21 as i32);
    }

    /// Testing support for an execute message where only the owner
    /// of the smart contract can set the score of an address.
    /// Case: Non-owner.
    #[test]
    fn set_by_anyone() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Execute Set as a non-owner.
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 21 as i32};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        // Check if the program errors.
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Ensure Address1's score is still 10.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);
    }

    /// Test support for a read query to get the score for a particular address.
    #[test]
    fn read_query_to_get_the_score_of_particular_address() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // EnsEnsure Address1's score is still 10.
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 10 as i32);
    }

    // ===========================
    // UNIT TESTS
    // ===========================

    /// Ensure one cannot set at an invalid address.
    #[test]
    fn set_by_owner_at_invalid_address() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Execute Set at an address that is not in our state HashMap.
        let info = mock_info("owner", &coins(1000, "earth"));
        let msg = ExecuteMsg::Set { address: "2".to_string(), new_score: 21 as i32};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        
        // Check if the program errors.
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must provide a valid address to set."),
        }
    }

    /// Ensure one cannot add an address if it already exists.
    #[test]
    fn error_if_adding_to_existing_address() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // Call AddAddress with an invalid address.
        let info = mock_info("owner", &coins(1000, "earth"));
        let new_address = "1".to_string();
        let new_score = 20 as i32;
        let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        // Check if the program errors.
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }
    }

    /// Testing storing scores for lots of addresses in the smart contract state.
    #[test]
    fn lots_of_addresses() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        for n in 2..47 {
            // Call AddAddress.
            let info = mock_info("owner", &coins(1000, "earth"));
            let new_address = n.to_string();
            let new_score = n * 10 as i32;
            let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
            let _res = execute(deps.as_mut(), mock_env(), info, msg);
        }

        for n in 1..47 {
            // Ensure Address{n}'s score is n * 10.
            let address = n.to_string();
            let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
            let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
            assert_eq!(value.score, n * 10 as i32);
        }
    }

    /// Testing storing scores for lots of addresses in the smart contract state.
    #[test]
    fn store_lots_of_addresses_then_set_them() {
        // Call the "Do Before Each" testing utility function.
        let (mut deps, info, msg) = setup();

        // Instantiate the contract.
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        for n in 2..20 {
            // Call AddAddress.
            let info = mock_info("owner", &coins(1000, "earth"));
            let new_address = n.to_string();
            let new_score = n * 10 as i32;
            let msg = ExecuteMsg::AddAddress { new_address: new_address, new_score: new_score};
            let _res = execute(deps.as_mut(), mock_env(), info, msg);
        }

        for n in 1..20 {
            // Ensure Address{n}'s score is n * 10.
            let address = n.to_string();
            let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : address}).unwrap();
            let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
            assert_eq!(value.score, n * 10 as i32);
        }

        for n in 1..20 {
            // Execute Set as owner.
            let info = mock_info("owner", &coins(1000, "earth"));
            let msg = ExecuteMsg::Set { address: n.to_string(), new_score: 100 as i32};
            let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }
        
        for n in 1..20 {
            // Make sure Address1's score is 100.
            let res = query(deps.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : n.to_string()}).unwrap();
            let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
            assert_eq!(value.score, 100 as i32);
        }
    }

    /// Testing owners of different smart contracts can't Set each other's scores.
    #[test]
    fn cross_set() {
        // Mock out dependencies.
        let mut deps1 = mock_dependencies(&[]);
        let mut deps2 = mock_dependencies(&[]);

        // Mock out account info.
        let info1 = mock_info("Alice", &coins(1000, "earth"));
        let info2 = mock_info("Bob", &coins(2, "token"));

        // Create a message that could be used to instantiate a smart contract.
        let msg1 = InstantiateMsg {
            first_address: "1".to_string(),
            first_address_score: 5 as i32
        };
        let msg2 = InstantiateMsg {
            first_address: "1".to_string(),
            first_address_score: 17 as i32
        };
    
        // Instantiate both smart contracts.
        let res1 = instantiate(deps1.as_mut(), mock_env(), info1, msg1).unwrap();
        assert_eq!(0, res1.messages.len());
        let res2 = instantiate(deps2.as_mut(), mock_env(), info2, msg2).unwrap();
        assert_eq!(0, res2.messages.len());

        // Reset the infos because they were moved.
        let info1 = mock_info("Alice", &coins(1000, "earth"));
        let info2 = mock_info("Bob", &coins(2, "token"));

        // Try to execute Set on Alice's contract as Bob.
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 0 as i32};
        let res = execute(deps1.as_mut(), mock_env(), info2, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Ensure Alice's Address1 score is still 5.
        let res = query(deps1.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 5 as i32);

        // Try to execute Set on Bob's contract as Alice.
        let msg = ExecuteMsg::Set { address: "1".to_string(), new_score: 0 as i32};
        let res = execute(deps2.as_mut(), mock_env(), info1, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // Ensure Bob's Address1 score is still 17.
        let res = query(deps2.as_ref(), mock_env(), QueryMsg::GetScoreFromAddress {address : "1".to_string()}).unwrap();
        let value: ScoreFromAddressResponse = from_binary(&res).unwrap();
        assert_eq!(value.score, 17 as i32);
    }
}
