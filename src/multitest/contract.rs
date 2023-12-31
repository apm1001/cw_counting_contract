use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, ContractWrapper, Executor};
use crate::migrate;
use crate::{execute, instantiate, query, msg::InstantiateMsg};
use cosmwasm_std::{Coin, StdResult};
use crate::error::ContractError;
use crate::msg::{ExecMsg, QueryMsg, ValueResp, Parent, MigrationMsg};

pub struct CountingContract(Addr);

impl CountingContract {
    pub fn addr(&self) -> &Addr {
        &self.0
    }

    pub fn store_code(app: &mut App) -> u64 {
        let contract = ContractWrapper::new(execute, instantiate, query).with_migrate(migrate);
        app.store_code(Box::new(contract))
    }

    #[track_caller]
    pub fn instantiate<'a>(
        app: &mut App,
        code_id: u64,
        sender: &Addr,
        label: &str,
        admin: impl Into<Option<&'a Addr>>,
        counter: impl Into<Option<u64>>,
        minimal_donation: Coin,
        parent: impl Into<Option<Parent>>
    ) -> StdResult<Self> {
        let admin = admin.into();
        let counter = counter.into().unwrap_or_default();
        app.instantiate_contract(
            code_id,
            sender.clone(),
            &InstantiateMsg {
                counter,
                minimal_donation,
                parent: parent.into()
            },
            &[],
            label,
            admin.map(Addr::to_string),
        )
        .map(CountingContract)
        .map_err(|err| err.downcast().unwrap())
    }

    #[track_caller]
    pub fn donate(
        &self,
        app: &mut App,
        sender: &Addr,
        funds: &[Coin],
    ) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Donate {}, funds)
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }

    #[track_caller]
    pub fn reset(
        &self,
        app: &mut App,
        sender: &Addr,
        counter: impl Into<Option<u64>>,
    ) -> Result<(), ContractError> {
        let counter = counter.into().unwrap_or_default();
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::Reset { new_value: counter },
            &[],
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn withdraw(&self, app: &mut App, sender: &Addr) -> Result<(), ContractError> {
        app.execute_contract(sender.clone(), self.0.clone(), &ExecMsg::Withdraw {}, &[])
            .map_err(|err| err.downcast().unwrap())
            .map(|_| ())
    }
 
    #[track_caller]
    pub fn withdraw_to(
        &self,
        app: &mut App,
        sender: &Addr,
        receiver: &Addr,
        funds: impl Into<Option<Vec<Coin>>>,
    ) -> Result<(), ContractError> {
        let funds = funds.into().unwrap_or_default();
        app.execute_contract(
            sender.clone(),
            self.0.clone(),
            &ExecMsg::WithdrawTo {
                recipient: receiver.to_string(),
                funds,
            },
            &[],
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| ())
    }

    #[track_caller]
    pub fn query_value(&self, app: &App) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Value {})
    }

    #[track_caller]
    pub fn query_incremented(&self, app: &App, value: u64) -> StdResult<ValueResp> {
        app.wrap()
            .query_wasm_smart(self.0.clone(), &QueryMsg::Incremented { value })
    }

    #[track_caller]
    pub fn migrate(
        app: &mut App, 
        contract: Addr, 
        code_id: u64, 
        sender: &Addr,
        parent: impl Into<Option<Parent>>
    ) -> StdResult<Self> {
        app.migrate_contract(
            sender.clone(), 
            contract.clone(), 
            &MigrationMsg {parent: parent.into()}, 
            code_id
        )
        .map_err(|err| err.downcast().unwrap())
        .map(|_| Self(contract))
    }
}

 
impl From<CountingContract> for Addr {
    fn from(contract: CountingContract) -> Self {
        contract.0
    }
}