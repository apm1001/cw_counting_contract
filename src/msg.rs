use cosmwasm_std::Coin;
use cosmwasm_schema::{cw_serde, QueryResponses};


#[cw_serde]
pub struct InstantiateMsg {
    #[serde(default)]
    pub counter: u64,
    pub minimal_donation: Coin,
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ValueResp)]
    Value {},
    #[returns(ValueResp)]
    Incremented {
        #[serde(default)]
        value: u64
    }
} 

#[cw_serde]
pub enum ExecMsg {
    Donate {},
    Reset { 
        #[serde(default)]
        new_value: u64 
    },
    Withdraw {},
    WithdrawTo {
        recipient: String,
        #[serde(default)]
        funds: Vec<Coin> 
    }
}
 
#[cw_serde]
pub struct ValueResp {
    pub value: u64,
}

