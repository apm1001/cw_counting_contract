use cosmwasm_std::{StdResult, DepsMut, Coin, Response};
use cw_storage_plus::Item;

use crate::state::{STATE, State};


pub fn migrate(deps: DepsMut) -> StdResult<Response> {
    const COUNTER: Item<u64> = Item::new("counter");
    const MINIMAL_DONATION: Item<Coin> = Item::new("minimal_donation");
 
    let counter = COUNTER.load(deps.storage)?;
    let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;
 
    STATE.save(
        deps.storage,
        &State {
            counter,
            minimal_donation,
        },
    )?;
 
    Ok(Response::new())
}

pub mod query {
    use cosmwasm_std::{Deps, StdResult, DepsMut, MessageInfo, Coin, Response};

    use crate::{msg::ValueResp, state::{State, STATE, OWNER}};

    pub fn instantiate(
        deps: DepsMut,
        info: MessageInfo, 
        counter: u64, 
        minimal_donation: Coin
    ) -> StdResult<Response> {
        STATE.save(
            deps.storage,
            &State {
                counter,
                minimal_donation,
            },
        )?;
        OWNER.save(deps.storage, &info.sender)?;
        Ok(Response::new())
    }
 
    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = STATE.load(deps.storage)?.counter;
        Ok(ValueResp { value })
    }

    pub fn incremented(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }

}

pub mod exec {
    
    use cosmwasm_std::{StdResult, DepsMut, Response, MessageInfo, Env, BankMsg, Coin, Uint128};
    
    use crate::{state::{STATE, OWNER}, error::ContractError};
 
    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut state = STATE.load(deps.storage)?;
 
        if state.minimal_donation.amount.is_zero()
            || info.funds.iter().any(|coin| {
                coin.denom == state.minimal_donation.denom
                    && coin.amount >= state.minimal_donation.amount
            })
        {
            state.counter += 1;
            STATE.save(deps.storage, &state)?;
        }
 
        let resp = Response::new()
            .add_attribute("action", "poke")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", state.counter.to_string());
 
        Ok(resp)
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, new_value: u64) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized { 
                owner: owner.to_string() 
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
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized { 
                owner: owner.to_string() 
            });
        }

        let balance = deps.querier.query_all_balances(&env.contract.address)?;
        let bank_msg = BankMsg::Send { 
            to_address: info.sender.to_string(), 
            amount: balance 
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
        funds: Vec<Coin>
    ) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender != owner {
            return Err(ContractError::Unauthorized { 
                owner: owner.to_string() 
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
            amount: balance
        };

        let resp = Response::new()
            .add_message(bank_msg)
            .add_attribute("action", "withdrawTo")
            .add_attribute("recipient", recipient);

        Ok(resp)
    }
}