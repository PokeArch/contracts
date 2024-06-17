use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Empty};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Pokemon {
    pub token_id: i32,
    pub index: i32,
    pub health: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Player {
    pub id: String,
    pub potions: i32,
    pub berries: i32,
    pub default_pokemon: i32,
    pub pokemons: Vec<Pokemon>,
}

pub const OWNER: Item<Addr> = Item::new("owner");

pub const ALLOWED_ADDRESSES: Map<Addr, Empty> = Map::new("allowed_addresses");

pub const NFT_CONTRACT: Item<Addr> = Item::new("nft_contract");

pub const PLAYERS: Map<String, Player> = Map::new("players");

pub const TOKEN: Item<i32> = Item::new("token");
