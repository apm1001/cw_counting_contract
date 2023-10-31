use cosmwasm_std::{
    entry_point, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

mod contract;
mod error;
pub mod msg;
mod state;

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

#[cfg(test)]
mod test {
    use crate::{
        execute, instantiate,
        msg::{ExecMsg, InstantiateMsg},
        query, error::ContractError,
    };

    use cosmwasm_std::{coin, coins, Addr, Empty};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    fn counting_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    use crate::msg::{QueryMsg, ValueResp};

    #[test]
    fn query_value() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let counter = 1;

        let minimal_donation = coin(10, "atom");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter,
                    minimal_donation,
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: counter });
    }

    #[test]
    fn query_incremented() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let counter = 1;

        let minimal_donation = coin(10, "atom");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter,
                    minimal_donation,
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Incremented { value: (1) })
            .unwrap();

        assert_eq!(resp, ValueResp { value: counter + 1 });
    }

    #[test]
    fn donate() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let counter = 0;
        let minimal_donation = coin(10, "atom");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter,
                    minimal_donation,
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &[],
        )
        .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: counter });
    }

    #[test]
    fn donate_with_funds() {
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());
        let counter = 0;
        let minimal_donation = coin(10, "atom");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter,
                    minimal_donation,
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
        .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: counter + 1 });
    }

    #[test]
    fn reset() {
        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());
        let counter = 1;
        let minimal_donation = coin(10, "atom");

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                Addr::unchecked("sender"),
                &InstantiateMsg {
                    counter,
                    minimal_donation,
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            Addr::unchecked("sender"),
            contract_addr.clone(),
            &ExecMsg::Reset { new_value: (10) },
            &[],
        )
        .unwrap();

        let resp: ValueResp = app
            .wrap()
            .query_wasm_smart(contract_addr, &QueryMsg::Value {})
            .unwrap();

        assert_eq!(resp, ValueResp { value: 10 });
    }

    #[test]
    fn withdraw() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("sender");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: coin(10, "atom"),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
        .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::Withdraw {},
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(owner).unwrap(),
            coins(10, "atom")
        );
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            vec![]
        );
    }

    #[test]
    fn unauthorized_withdraw() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: coin(10, "atom"),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(member, contract_addr, &ExecMsg::Withdraw {}, &[])
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: owner.into()
            },
            err.downcast().unwrap()
        );
    }

    #[test]
    fn withdraw_to() {
        let owner = Addr::unchecked("owner");
        let sender = Addr::unchecked("sender");
        let recipient = Addr::unchecked("recipient");

        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(10, "atom"))
                .unwrap();
        });

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: coin(10, "atom"),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        app.execute_contract(
            sender.clone(),
            contract_addr.clone(),
            &ExecMsg::Donate {},
            &coins(10, "atom"),
        )
        .unwrap();

        app.execute_contract(
            owner.clone(),
            contract_addr.clone(),
            &ExecMsg::WithdrawTo {
                recipient: recipient.to_string(),
                funds: coins(5, "atom"),
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap().query_all_balances(recipient).unwrap(),
            coins(5, "atom")
        );
        assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
        assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
        assert_eq!(
            app.wrap().query_all_balances(contract_addr).unwrap(),
            coins(5, "atom")
        );
    }

    #[test]
    fn unauthorized_withdraw_to() {
        let owner = Addr::unchecked("owner");
        let member = Addr::unchecked("member");

        let mut app = App::default();

        let contract_id = app.store_code(counting_contract());

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                owner.clone(),
                &InstantiateMsg {
                    counter: 0,
                    minimal_donation: coin(10, "atom"),
                },
                &[],
                "Counting contract",
                None,
            )
            .unwrap();

        let err = app
            .execute_contract(
                member, 
                contract_addr, 
                &ExecMsg::WithdrawTo { 
                    recipient: owner.to_string(), 
                    funds: coins(10, "atom") 
                }, 
                &[]
            )
            .unwrap_err();

        assert_eq!(
            ContractError::Unauthorized {
                owner: owner.into()
            },
            err.downcast().unwrap()
        );
    }
}
