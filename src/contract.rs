#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    ensure, to_json_binary, wasm_execute, BankMsg, Binary, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdResult,
};
use cw2::set_contract_version;
use cw_utils::one_coin;
use kujira::{KujiraMsg, KujiraQuery};

use crate::msg::Config;
use crate::state::CONFIG;
use crate::{ContractError, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

const CONTRACT_NAME: &str = "fuzion/ghost-vaults-swap-adapter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn migrate(deps: DepsMut<KujiraQuery>, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut<KujiraQuery>,
    _: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: msg.owner,
        unstakefi_config: msg.unstakefi_config,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<KujiraQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<KujiraMsg>, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    match msg {
        ExecuteMsg::UpdateConfig {
            owner,
            unstakefi_config,
        } => {
            ensure!(info.sender == config.owner, ContractError::Unauthorized {});
            if let Some(owner) = owner {
                config.owner = owner;
            }
            if let Some(unstakefi_config) = unstakefi_config {
                config.unstakefi_config = unstakefi_config;
            }
            CONFIG.save(deps.storage, &config)?;
            Ok(Response::default())
        }
        ExecuteMsg::Swap { callback, .. } => {
            let received = one_coin(&info)?;

            let config = CONFIG.load(deps.storage)?;

            let denom_config = config
                .unstakefi_config
                .iter()
                .find(|x| x.denom.to_string() == received.denom);

            let msg = if let Some(denom_config) = denom_config {
                let msg = unstake::controller::ExecuteMsg::Fund {};
                let addr = &denom_config.address;
                wasm_execute(addr, &msg, info.funds)?
            } else {
                ensure!(
                    received.denom.starts_with("factory/"),
                    ContractError::InvalidDenom(received.denom)
                );

                let split = received.denom.split('/').collect::<Vec<&str>>();
                ensure!(
                    split.len() == 3,
                    ContractError::InvalidDenom(received.denom)
                );
                ensure!(
                    split[2] == "urcpt",
                    ContractError::InvalidDenom(received.denom)
                );

                let msg = unstake::controller::ExecuteMsg::Withdraw {};
                let addr = split[1];
                wasm_execute(addr, &msg, info.funds)?
            };
            let post_swap_msg = wasm_execute(
                env.contract.address,
                &ExecuteMsg::PostSwap {
                    callback,
                    sender: info.sender,
                },
                vec![],
            )?;
            Ok(Response::new().add_message(msg).add_message(post_swap_msg))
        }
        ExecuteMsg::PostSwap { callback, sender } => {
            ensure!(
                info.sender == env.contract.address,
                ContractError::Unauthorized {}
            );
            let funds = deps.querier.query_all_balances(env.contract.address)?;

            let return_msg = match callback {
                Some(callback) => callback.to_message(&sender, Empty {}, funds)?,
                None => BankMsg::Send {
                    to_address: sender.to_string(),
                    amount: funds,
                }
                .into(),
            };

            Ok(Response::new().add_message(return_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<KujiraQuery>, _: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    Ok(match msg {
        QueryMsg::Config {} => to_json_binary(&config),
    }?)
}
