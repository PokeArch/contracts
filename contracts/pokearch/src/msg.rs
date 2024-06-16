use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Player;

#[cw_serde]
pub struct InstantiateMsg {
    pub count: i32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Increment {},
    Reset { count: i32 },
    RemoveAllowance(String),
    AddAllowance(String),
    SetNFTContract{ addr: String, token_uri: String },
    Register { id: String },
    CatchPokemon { id: String, token_uri: String },
    UpdateHealth { id: String, token_id: i32 },
    CollectBerries { id: String },
    SetDefaultPokemon { id: String, pokemon: i32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetCount {},
    #[returns(bool)]
    CheckAllowance { addr: String },
    #[returns(PlayerResponse)]
    GetPlayer { id: String }
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub count: i32,
}

#[cw_serde]
pub struct PlayerResponse {
    pub player: Player
}