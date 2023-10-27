 
pub mod query {
    use cosmwasm_std::{Deps, StdResult, Response, DepsMut, Coin};

    use crate::{msg::*, state::{COUNTER, MINIMAL_DONATION}};

    pub fn instantiate(deps: DepsMut, counter: u64, minimal_donation: Coin) -> StdResult<Response> {
        COUNTER.save(deps.storage, &counter)?;
        MINIMAL_DONATION.save(deps.storage, &minimal_donation)?;
        Ok(Response::new())
    }
 
    pub fn value(deps: Deps) -> StdResult<ValueResp> {
        let value = COUNTER.load(deps.storage)?;
        Ok(ValueResp { value })
    }

    pub fn incremented(value: u64) -> ValueResp {
        ValueResp { value: value + 1 }
    }

}

pub mod exec {
    use cosmwasm_std::{DepsMut, Response, StdResult, MessageInfo};
 
    use crate::state::{COUNTER, MINIMAL_DONATION};
 
    pub fn donate(deps: DepsMut, info: MessageInfo) -> StdResult<Response> {
        let mut counter = COUNTER.load(deps.storage)?;
        let minimal_donation = MINIMAL_DONATION.load(deps.storage)?;

        if info.funds.iter().any(|coin| {
            coin.denom == minimal_donation.denom && coin.amount >= minimal_donation.amount
        }) {
            counter += 1;
            COUNTER.save(deps.storage, &counter)?;
        }
 
        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("counter", counter.to_string());
 
        Ok(resp)
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, new_value: u64) -> StdResult<Response> {
        COUNTER.save(deps.storage, &new_value)?;
 
        let resp = Response::new()
            .add_attribute("action", "donate")
            .add_attribute("sender", info.sender.as_str())
            .add_attribute("new_value", new_value.to_string());
 
        Ok(resp)
    }
}