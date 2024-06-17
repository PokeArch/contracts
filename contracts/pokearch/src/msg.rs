use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::state::Player;

#[cw_serde]
pub struct InstantiateMsg { }

#[cw_serde]
pub enum ExecuteMsg {
    RemoveAllowance(String),
    AddAllowance(String),
    SetNFTContract{ addr: String, token_uri: String },
    Register { id: String },
    CatchPokemon { id: String, token_uri: String, health: i32, curr_pokemon: i32 },
    UpdateHealth { id: String, token_id: i32 },
    CollectBerries { id: String },
    SetDefaultPokemon { id: String, pokemon: i32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(bool)]
    CheckAllowance { addr: String },
    #[returns(PlayerResponse)]
    GetPlayer { id: String }
}

#[cw_serde]
pub struct PlayerResponse {
    pub player: Player
}