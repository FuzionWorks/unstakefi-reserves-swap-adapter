//! This contract provides an interface to swap Unstake.Fi receipt assets.
//! The interface is compatible with the FIN "swap" message, so others can
//! directly plug into it
pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
pub use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
