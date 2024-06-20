use andromeda_non_fungible_tokens::cw721::ExecuteMsg as Cw721ExecuteMsg;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use cw2::set_contract_version;

use crate::cwfees::{CwGrant, MsgRegisterAsGranter, SudoMsg};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ALLOWED_ADDRESSES, NFT_CONTRACT, OWNER, PLAYERS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pokearch";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let contract_address = env.clone().contract.address;

    OWNER.save(deps.storage, &info.sender)?;
    ALLOWED_ADDRESSES.save(deps.storage, info.sender.clone(), &Empty {})?;

    let contract_address = contract_address.to_string();
    let regsiter_msg = MsgRegisterAsGranter {
        granting_contract: contract_address.clone(),
    };
    let register_stargate_msg = CosmosMsg::Stargate {
        type_url: "/archway.cwfees.v1.MsgRegisterAsGranter".to_string(),
        value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
    };

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("action", "register")
        .add_message(register_stargate_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RemoveAllowance(addr) => {
            ALLOWED_ADDRESSES.remove(deps.storage, deps.api.addr_validate(&addr)?);
            Ok(Response::default())
        }
        ExecuteMsg::AddAllowance(addr) => {
            ALLOWED_ADDRESSES.save(
                deps.storage,
                deps.api.addr_validate(&addr)?,
                &Empty::default(),
            )?;
            Ok(Response::default())
        }
        ExecuteMsg::SetNFTContract { addr, token_uri } => {
            execute::set_nft_address(deps, info, env, addr, token_uri)
        }
        ExecuteMsg::Register { id } => execute::register(deps, id),
        ExecuteMsg::CatchPokemon {
            id,
            token_uri,
            health,
            curr_pokemon,
        } => execute::catch_pokemon(deps, info, env, id, token_uri, health, curr_pokemon),
        ExecuteMsg::UpdateHealth { id, token_id } => execute::update_health(deps, id, token_id),
        ExecuteMsg::CollectBerries { id } => execute::collect_berries(deps, id),
        ExecuteMsg::SetDefaultPokemon { id, pokemon } => {
            execute::set_default_pokemon(deps, id, pokemon)
        }
    }
}

pub mod execute {
    use andromeda_non_fungible_tokens::cw721::TokenExtension;

    use crate::state::{Player, Pokemon, TOKEN};

    use super::*;

    pub fn set_nft_address(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        addr: String,
        token_uri: String,
    ) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender.to_string() == owner.to_string() {
            let mint: Cw721ExecuteMsg = Cw721ExecuteMsg::Mint {
                token_id: 0.to_string(),
                owner: env.contract.address.to_string(),
                token_uri: Some(token_uri),
                extension: TokenExtension {
                    publisher: "PokeArch".to_string(),
                },
            };

            let wasm_msg = WasmMsg::Execute {
                contract_addr: addr.clone(),
                msg: to_json_binary(&mint)?,
                funds: Vec::new(),
            };
            NFT_CONTRACT.save(deps.storage, &deps.api.addr_validate(&addr.clone())?)?;
            TOKEN.save(deps.storage, &(0 as i32))?;
            Ok(Response::new().add_message(wasm_msg))
        } else {
            return Err(ContractError::Unauthorized {});
        }
    }

    pub fn register(deps: DepsMut, id: String) -> Result<Response, ContractError> {
        if PLAYERS.has(deps.storage, id.clone()) {
            return Err(ContractError::Unauthorized {});
        }
        let mut pokemon: Vec<Pokemon> = Vec::new();
        pokemon.push(Pokemon {
            token_id: 0,
            index: 0,
            health: 100,
        });

        let player_data = Player {
            id: id.clone(),
            potions: 0,
            berries: 0,
            default_pokemon: 0,
            pokemons: pokemon,
        };
        PLAYERS.save(deps.storage, id.clone(), &player_data)?;
        Ok(Response::default())
    }

    pub fn update_health(
        deps: DepsMut,
        id: String,
        token_id: i32,
    ) -> Result<Response, ContractError> {
        let mut player = PLAYERS.load(deps.storage, id.clone())?;
        player.pokemons[token_id as usize].health = 100;
        PLAYERS.save(deps.storage, id.clone(), &player)?;
        Ok(Response::default())
    }

    pub fn collect_berries(deps: DepsMut, id: String) -> Result<Response, ContractError> {
        let mut player = PLAYERS.load(deps.storage, id.clone())?;
        player.berries += 1;
        PLAYERS.save(deps.storage, id.clone(), &player)?;
        Ok(Response::default())
    }

    pub fn set_default_pokemon(
        deps: DepsMut,
        id: String,
        pokemon: i32,
    ) -> Result<Response, ContractError> {
        let mut player = PLAYERS.load(deps.storage, id.clone())?;
        player.default_pokemon = pokemon;
        PLAYERS.save(deps.storage, id.clone(), &player)?;
        Ok(Response::default())
    }

    pub fn catch_pokemon(
        deps: DepsMut,
        info: MessageInfo,
        env: Env,
        id: String,
        token_uri: String,
        health: i32,
        curr_pokemon: i32,
    ) -> Result<Response, ContractError> {
        let token = TOKEN.load(deps.storage)?;
        let nft_address = NFT_CONTRACT.load(deps.storage)?;

        let mint: Cw721ExecuteMsg = Cw721ExecuteMsg::Mint {
            token_id: (token + 1).to_string(),
            owner: info.sender.clone().to_string(),
            token_uri: Some(token_uri),
            extension: TokenExtension {
                publisher: "PokeArch".to_string(),
            },
        };

        let wasm_msg = WasmMsg::Execute {
            contract_addr: nft_address.clone().to_string(),
            msg: to_json_binary(&mint)?,
            funds: Vec::new(),
        };

        let mut player = PLAYERS.load(deps.storage, id.clone())?;
        player.pokemons.push(Pokemon {
            token_id: token + 1,
            index: (player.pokemons.len() as i32),
            health: 100,
        });
        player.pokemons[curr_pokemon as usize].health = health;

        PLAYERS.save(deps.storage, id.clone(), &player)?;
        TOKEN.save(deps.storage, &(token + 1))?;
        Ok(Response::new().add_message(wasm_msg))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CheckAllowance { addr } => to_json_binary(&query::check_allowance(deps, addr)?),
        QueryMsg::GetPlayer { id } => to_json_binary(&query::get_player(deps, id)?),
    }
}

pub mod query {

    use crate::msg::PlayerResponse;

    use super::*;

    pub fn check_allowance(deps: Deps, addr: String) -> StdResult<bool> {
        Ok(ALLOWED_ADDRESSES.has(deps.storage, deps.api.addr_validate(&addr)?))
    }
    pub fn get_player(deps: Deps, id: String) -> StdResult<PlayerResponse> {
        let player = PLAYERS.load(deps.storage, id)?;
        Ok(PlayerResponse { player })
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    return match msg {
        SudoMsg::CwGrant(grant) => process_grant(deps, grant),
        _ => Err(ContractError::Unauthorized {}),
    };
}

fn process_grant(deps: DepsMut, grant: CwGrant) -> Result<Response, ContractError> {
    const TYPE_URL: &str = "cosmwasm.wasm.v1.MsgExecuteContract";

    for msg in grant.msgs {
        // we check if all the senders are in the allow list
        let addr = deps.api.addr_validate(&msg.sender)?;
        if !ALLOWED_ADDRESSES.has(deps.storage, addr) {
            return Err(ContractError::Unauthorized {});
        }

        // we check the message type url
        if msg.type_url != TYPE_URL {
            return Err(ContractError::DisallowedMessage(msg.type_url));
        }
    }

    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use std::alloc::System;

    use crate::msg::PlayerResponse;
    use crate::state::{Player, Pokemon};

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json, Addr};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CheckAllowance {
                addr: Addr::unchecked("creator").to_string(),
            },
        )
        .unwrap();
        let value: bool = from_json(&res).unwrap();
        assert_eq!(true, value);
    }

    #[test]
    fn set_nft_address() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetNFTContract {
            addr: Addr::unchecked("nft").to_string(),
            token_uri: String::from("hello"),
        };
        let info = mock_info("creator", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
    }

    #[test]
    fn add_allowance() {
        let mut deps = mock_dependencies();
        let msg = ExecuteMsg::AddAllowance(Addr::unchecked("sender").to_string());
        let info = mock_info("sender", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::CheckAllowance {
                addr: Addr::unchecked("sender").to_string(),
            },
        )
        .unwrap();
        let value: bool = from_json(&res).unwrap();
        assert_eq!(true, value);
    }

    #[test]
    fn register() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Register {
            id: "hello.arch".to_string(),
        };
        let info = mock_info("sender", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mut pokemon: Vec<Pokemon> = Vec::new();
        pokemon.push(Pokemon {
            token_id: 0,
            index: 0,
            health: 100,
        });

        let player_data = Player {
            id: String::from("hello.arch"),
            potions: 0,
            berries: 0,
            default_pokemon: 0,
            pokemons: pokemon,
        };

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPlayer {
                id: String::from("hello.arch"),
            },
        )
        .unwrap();
        let value: PlayerResponse = from_json(&res).unwrap();
        assert_eq!(
            PlayerResponse {
                player: player_data
            },
            value
        );
    }

    #[test]
    fn test_game() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetNFTContract {
            addr: "nft".to_string(),
            token_uri: String::from("hello"),
        };
        let info = mock_info("creator", &[]);

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Register {
            id: String::from("hello.arch"),
        };
        let info = mock_info("sender", &[]);

        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::CatchPokemon {
            id: String::from("hello.arch"),
            token_uri: String::from("hello"),
            health: 32,
            curr_pokemon: 0,
        };
        let info = mock_info("sender", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());

        let mut pokemon: Vec<Pokemon> = Vec::new();
        pokemon.push(Pokemon {
            token_id: 0,
            index: 0,
            health: 32,
        });

        pokemon.push(Pokemon {
            token_id: 1,
            index: 1,
            health: 100,
        });

        let player_data = Player {
            id: String::from("hello.arch"),
            potions: 0,
            berries: 0,
            default_pokemon: 0,
            pokemons: pokemon,
        };

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPlayer {
                id: String::from("hello.arch"),
            },
        )
        .unwrap();
        let value: PlayerResponse = from_json(&res).unwrap();
        assert_eq!(
            PlayerResponse {
                player: player_data
            },
            value
        );

        let msg = ExecuteMsg::UpdateHealth {
            id: String::from("hello.arch"),
            token_id: 0,
        };
        let info = mock_info("sender", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let mut pokemon: Vec<Pokemon> = Vec::new();
        pokemon.push(Pokemon {
            token_id: 0,
            index: 0,
            health: 100,
        });

        pokemon.push(Pokemon {
            token_id: 1,
            index: 1,
            health: 100,
        });

        let player_data = Player {
            id: String::from("hello.arch"),
            potions: 0,
            berries: 0,
            default_pokemon: 0,
            pokemons: pokemon,
        };

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetPlayer {
                id: String::from("hello.arch"),
            },
        )
        .unwrap();
        let value: PlayerResponse = from_json(&res).unwrap();
        assert_eq!(
            PlayerResponse {
                player: player_data
            },
            value
        );
        let msg = ExecuteMsg::CatchPokemon {
            id: String::from("hello.arch"),
            token_uri: String::from("hello"),
            health: 32,
            curr_pokemon: 0,
        };
        let info = mock_info("sender", &[]);

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());

    }
}
