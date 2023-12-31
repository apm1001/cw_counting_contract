use cosmwasm_std::{coin, coins, Addr, Decimal};
use cw_multi_test::App;
 
use crate::{msg::{ValueResp, Parent}, error::ContractError, state::{STATE, State, PARENT_DONATION, ParentDonation}};
use super::contract::CountingContract;
use counting_contract_0_3::multitest::contract::CountingContract as CountingContract_0_3;

 
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
        None,
        coin(10, ATOM),
        None
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
 
#[test]
fn migration() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });
 
    let old_code_id = CountingContract_0_3::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);
 
    let contract = CountingContract_0_3::instantiate(
        &mut app,
        old_code_id,
        &owner,
        "Counting contract",
        &admin,
        None,
        coin(10, ATOM),
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();
 
    let contract =
        CountingContract::migrate(
            &mut app, 
            contract.into(), 
            new_code_id, 
            &admin,
            None
        ).unwrap();
 
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });
 
    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM),
            owner, 
            donating_parent: None
        }
    );
}

#[test]
fn migration_same_version() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        &admin,
        None,
        coin(10, ATOM),
        None
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let contract = CountingContract::migrate(
        &mut app, 
        contract.into(), 
        code_id, 
        &admin, 
        None
    ).unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM),
            owner,
            donating_parent: None
        }
    );
}

#[test]
fn donating_parent() {
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
 
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(20, "atom"))
            .unwrap();
    });
 
    let code_id = CountingContract::store_code(&mut app);
 
    let parent_contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Parent contract",
        None,
        None,
        coin(0, ATOM),
        None,
    )
    .unwrap();
 
    let contract = CountingContract::instantiate(
        &mut app,
        code_id,
        &owner,
        "Counting contract",
        None,
        None,
        coin(10, ATOM),
        Parent {
            addr: parent_contract.addr().to_string(),
            donating_period: 2,
            part: Decimal::percent(10),
        },
    )
    .unwrap();
 
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();
    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();
 
    let resp = parent_contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });
 
    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 2 });
 
    assert_eq!(app.wrap().query_all_balances(owner).unwrap(), vec![]);
    assert_eq!(app.wrap().query_all_balances(sender).unwrap(), vec![]);
    assert_eq!(
        app.wrap().query_all_balances(contract.addr()).unwrap(),
        coins(18, ATOM)
    );
    assert_eq!(
        app.wrap()
            .query_all_balances(parent_contract.addr())
            .unwrap(),
        coins(2, ATOM)
    );
}

#[test]
fn migration_with_parent() {
    let admin = Addr::unchecked("admin");
    let owner = Addr::unchecked("owner");
    let sender = Addr::unchecked("sender");
    let parent = Addr::unchecked("parent");

    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &sender, coins(10, "atom"))
            .unwrap();
    });

    let old_code_id = CountingContract_0_3::store_code(&mut app);
    let new_code_id = CountingContract::store_code(&mut app);

    let contract = CountingContract_0_3::instantiate(
        &mut app,
        old_code_id,
        &owner,
        "Counting contract",
        &admin,
        None,
        coin(10, ATOM),
    )
    .unwrap();

    contract
        .donate(&mut app, &sender, &coins(10, ATOM))
        .unwrap();

    let contract = CountingContract::migrate(
        &mut app,
        contract.into(),
        new_code_id,
        &admin,
        Parent {
            addr: parent.to_string(),
            donating_period: 2,
            part: Decimal::percent(10),
        },
    )
    .unwrap();

    let resp = contract.query_value(&app).unwrap();
    assert_eq!(resp, ValueResp { value: 1 });

    let state = STATE.query(&app.wrap(), contract.addr().clone()).unwrap();
    assert_eq!(
        state,
        State {
            counter: 1,
            minimal_donation: coin(10, ATOM),
            owner,
            donating_parent: Some(2),
        }
    );

    let parent_donation = PARENT_DONATION
        .query(&app.wrap(), contract.addr().clone())
        .unwrap();
    assert_eq!(
        parent_donation,
        ParentDonation {
            address: parent,
            donating_parent_period: 2,
            part: Decimal::percent(10),
        }
    )
}