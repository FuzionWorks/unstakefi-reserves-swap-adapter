use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal256};
use kujira::{CallbackData, Denom};

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: Addr,
    pub unstakefi_config: Vec<UnstakeFiConfig>,
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Option<Addr>,
        unstakefi_config: Option<Vec<UnstakeFiConfig>>,
    },
    /* Compatibility with FIN */
    Swap {
        /// Field provided for backward compatibility but ignored. Only a single
        /// asset may be provided for a swap
        offer_asset: Option<Coin>,
        belief_price: Option<Decimal256>,
        max_spread: Option<Decimal256>,
        to: Option<Addr>,
        /// An optional callback that FIN will execute with the funds from the swap.
        /// The callback is executed on the sender's address.
        #[serde(skip_serializing_if = "Option::is_none")]
        callback: Option<CallbackData>,
    },
    /// Internal use -- returns the withdrawn funds to the sender
    PostSwap {
        callback: Option<CallbackData>,
        sender: Addr,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
}

#[cw_serde]
pub struct UnstakeFiConfig {
    pub address: Addr,
    pub denom: Denom,
}

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub unstakefi_config: Vec<UnstakeFiConfig>,
}
