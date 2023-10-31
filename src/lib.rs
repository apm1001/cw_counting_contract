use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

mod contract;
mod error;
pub mod msg;
mod state;
#[cfg(test)]
pub mod multitest;

use error::ContractError;
use msg::{ExecMsg, InstantiateMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    contract::query::instantiate(deps, info, msg.counter, msg.minimal_donation)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: msg::QueryMsg) -> StdResult<Binary> {
    use contract::query;
    use msg::QueryMsg::*;

    match msg {
        Value {} => to_binary(&query::value(deps)?),
        Incremented { value } => to_binary(&query::incremented(value)),
    }
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecMsg,
) -> Result<Response, ContractError> {
    use contract::exec;
    use msg::ExecMsg::*;

    match msg {
        Donate {} => exec::donate(deps, info).map_err(ContractError::Std),
        Reset { new_value } => exec::reset(deps, info, new_value),
        Withdraw {} => exec::withdraw(deps, env, info),
        WithdrawTo { recipient, funds } => exec::withdraw_to(deps, env, info, recipient, funds),
    }
}
