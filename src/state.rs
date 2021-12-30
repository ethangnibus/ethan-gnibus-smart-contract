// ======================================================================
// Imports
// ======================================================================

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// ======================================================================
// State Block
// ======================================================================

/// Create a struct to represent a state in a smart contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    /// The name of the owner of the smart contract.
    pub owner: Addr,

    /// A HashMap of addresses and cooresponding scores converted to a JSON String.
    pub hash: String,
}

// Make a constant State to save states (see: contract.rs).
pub const STATE: Item<State> = Item::new("state");
