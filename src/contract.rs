use crate::{
    msg::Parent,
    state::{ParentDonation, State, PARENT_DONATION, STATE},
};
use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response, StdResult};

use cw2::set_contract_version;
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
    info: MessageInfo,
    counter: u64,
    minimal_donation: Coin,
    parent: Option<Parent>,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
            owner: info.sender,
            donating_parent: parent.as_ref().map(|p| p.donating_period),
        },
    )?;

    if let Some(parent) = parent {
        PARENT_DONATION.save(
            deps.storage,
            &ParentDonation {
                address: deps.api.addr_validate(&parent.addr)?,
                donating_parent_period: parent.donating_period,
                part: parent.part,
            },
        )?;
    }
    Ok(Response::new())
}

pub mod migration {

    use crate::{error::ContractError, state::{self, PARENT_DONATION, ParentDonation}, msg::Parent};
    use cosmwasm_std::{Addr, Coin, DepsMut, Response, StdResult};
    use cw2::{get_contract_version, set_contract_version};
    use cw_storage_plus::Item;
    use serde::{Deserialize, Serialize};

    use super::{CONTRACT_NAME, CONTRACT_VERSION};

    pub fn migrate(mut deps: DepsMut, parent: Option<Parent>) -> Result<Response, ContractError> {
        
        let contract_version = get_contract_version(deps.storage)?;

        
        if contract_version.contract != CONTRACT_NAME {
            return Err(ContractError::InvalidContract {
                contract: contract_version.contract,
            });
        }
        
        // print!("{:?}", contract_version);

        let resp = match contract_version.version.as_str() {
            "0.2.0" => migrate_0_2_0(deps.branch()).map_err(ContractError::from)?,
            "0.3.0" => migrate_0_3_0(deps.branch(), parent).map_err(ContractError::from)?,
            CONTRACT_VERSION => return Ok(Response::default()),
            version => {
                return Err(ContractError::InvalidContractVersion {
                    version: version.into(),
                })
            }
        };
        set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

        Ok(resp)
    }

    pub fn migrate_0_2_0(deps: DepsMut) -> StdResult<Response> {
        #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
        struct State {
            pub counter: u64,
            pub minimal_donation: Coin,
        }

        const OLD_STATE: Item<State> = Item::new("state");
        const OWNER: Item<Addr> = Item::new("owner");

        let owner = OWNER.load(deps.storage)?;
        let old_state = OLD_STATE.load(deps.storage)?;

        let counter = old_state.counter;
        let minimal_donation = old_state.minimal_donation;

        state::STATE.save(
            deps.storage,
            &state::State {
                counter,
                minimal_donation,
                owner,
                donating_parent: None,
            },
        )?;

        Ok(Response::new())
    }

    pub fn migrate_0_3_0(deps: DepsMut, parent: Option<Parent>) -> StdResult<Response> {
        #[derive(Serialize, Deserialize)]
        struct OldState {
            counter: u64,
            minimal_donation: Coin,
            owner: Addr,
        }

        const OLD_STATE: Item<OldState> = Item::new("state");

        let OldState {
            counter,
            minimal_donation,
            owner,
        } = OLD_STATE.load(deps.storage)?;

        state::STATE.save(
            deps.storage,
            &state::State {
                counter,
                minimal_donation,
                owner,
                donating_parent: parent.as_ref().map(|p| p.donating_period),
            },
        )?;

        if let Some(parent) = parent {
            PARENT_DONATION.save(
                deps.storage,
                &ParentDonation {
                    address: deps.api.addr_validate(&parent.addr)?,
                    donating_parent_period: parent.donating_period,
                    part: parent.part,
                },
            )?;
        }

        Ok(Response::new())
    }
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult};

    use crate::{msg::ValueResp, state::STATE};

    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp { value })
    }

    pub fn incremented(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }
}

pub mod exec {

    use cosmwasm_std::{
        to_binary, BankMsg, Coin, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,
    };

    use crate::{
        error::ContractError,
        msg::ExecMsg,
        state::{PARENT_DONATION, STATE},
    };

    pub fn donate(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
        let mut resp = Response::new();

        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;

            if let Some(parent) = &mut state.donating_parent {
                *parent -= 1;

                if *parent == 0 {
                    let parent_donation = PARENT_DONATION.load(deps.storage)?;
                    *parent = parent_donation.donating_parent_period;

                    let funds: Vec<_> = deps
                        .querier
                        .query_all_balances(env.contract.address)?
                        .into_iter()
                        .map(|mut coin| {
                            coin.amount = coin.amount * parent_donation.part;
                            coin
                        })
                        .collect();

                    let msg = WasmMsg::Execute {
                        contract_addr: parent_donation.address.to_string(),
                        msg: to_binary(&ExecMsg::Donate {})?,
                        funds,
                    };

                    resp = resp
                        .add_message(msg)
                        .add_attribute("donated_to_parent", parent_donation.address.to_string());
                }
            }

            STATE.save(deps.storage, &state)?;
        }

        resp = resp
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());

        Ok(resp)
    }

    pub fn reset(
        deps: DepsMut,
        info: MessageInfo,
        new_value: u64,
    ) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        STATE.update(deps.storage, |mut state| -> StdResult<_> {
            state.counter = new_value;
            Ok(state)
        })?;

        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("new_value", new_value.to_string());

        Ok(resp)
    }

    pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdraw")
            .add_attribute("sender", info.sender.as_str());

        Ok(resp)
    }

    pub fn withdraw_to(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        recipient: String,
        funds: Vec<Coin>,
    ) -> Result<Response, ContractError> {
        let owner = STATE.load(deps.storage)?.owner;
        if info.sender != owner {
            return Err(ContractError::Unauthorized {
                owner: owner.to_string(),
            });
        }

        let mut balance = deps.querier.query_all_balances(&env.contract.address)?;

        if !funds.is_empty() {
            for coin in &mut balance {
                let limit = funds
                    .iter()
                    .find(|c| c.denom == coin.denom)
                    .map(|c| c.amount)
                    .unwrap_or(Uint128::zero());

                coin.amount = std::cmp::min(coin.amount, limit);
            }
        }

        let bank_msg = BankMsg::Send {
            to_address: recipient.clone(),
            amount: balance,
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdrawTo")
            .add_attribute("recipient", recipient);

        Ok(resp)
    }
}
