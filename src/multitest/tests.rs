use cosmwasm_std::{coin, coins, Addr};
use cw_multi_test::App;
 
use crate::{msg::ValueResp, error::ContractError};
 
use super::contract::CountingContract;
 
const ATOM: &str = "atom";
 

#[test]
fn query_value() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
 
    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
  
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 0 });
}

#[test]
fn query_incremented() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
 
    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
  
    let resp = contract.query_incremented(&app, 1).unwrap();
    assert_eq!(resp, ValueResp { value: 2 });
}

#[test]
fn donate() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &owner, &[])
        .unwrap();
 
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 0 });
}

#[test]
fn donate_with_funds() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();
 
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });
}

#[test]
fn reset() {
    let owner = Addr::unchecked("owner");
    let mut app = App::default();
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .reset(&mut app, &owner, 10)
        .unwrap();
 
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 10 });
}

#[test]
fn withdraw() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    contract.withdraw(&mut app, &owner).unwrap();
 
    assert_eq!(
        app.wrap().query_all_balances(owner).unwrap(),
        coins(10, ATOM)
    );
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        vec![]
    );
}

#[test]
fn unauthorized_withdraw() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let err = contract.withdraw(&mut app, &sender).unwrap_err();
 
    assert_eq!(
        ContractError::Unauthorized {
            owner: owner.into()
        },
        err
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
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    contract.withdraw_to(
        &mut app, 
        &owner, 
        &recipient,
        Some(coins(5, ATOM))
    ).unwrap();
 
    assert_eq!(
        app.wrap().query_all_balances(recipient).unwrap(),
        coins(5, ATOM)
    );
    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(5, ATOM)
    );
}

#[test]
fn unauthorized_withdraw_to() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, ATOM))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let err = contract.withdraw_to(
        &mut app, 
        &sender, 
        &owner, 
        Some(coins(5, ATOM))
    ).unwrap_err();
 
    assert_eq!(
        ContractError::Unauthorized {
            owner: owner.into()
        },
        err
    );
}
