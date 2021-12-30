// ======================================================================
// Imports
// ======================================================================

use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ======================================================================
// Message Block
// ======================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The smart contract's initial address.
    pub first_address: String,

    /// The score cooresponding to the smart contract's initial address.
    pub first_address_score: i32 ,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Outline the blueprint for a ExecuteMsg::Set(...).
    Set { address: String, new_score: i32 },

    /// Outline the blueprint for a ExecuteMsg::AddAddress(...).
    AddAddress { new_address: String, new_score: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Outline the blueprint for a QueryMsg::GetOwner().
    GetOwner {},

    /// Outline the blueprint for a QueryMsg::GetHash().
    GetHash {},

    /// Outline the blueprint for a QueryMsg::GetScoreFromAddress(...).
    GetScoreFromAddress { address: String },
}

// ======================================================================
// Response Block
// ======================================================================

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
    /// The name of the smart contract's owner.
    pub owner: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HashResponse {
    /// A HashMap of addresses and cooresponding scores converted to a JSON String.
    pub hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ScoreFromAddressResponse {
    /// The score from a corresponding address in the state HashMap.
    pub score: i32,
}
