use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult, WasmMsg};
use cw2::set_contract_version;


use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::cwfees::{SudoMsg, MsgRegisterAsGranter, CwGrant};
use crate::state::{State, STATE, ALLOWED_ADDRESSES, OWNER, NFT_CONTRACT, PLAYERS};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:pokearch";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    let contract_address = env.clone().contract.address;

    OWNER.save(deps.storage, &info.sender)?;
    ALLOWED_ADDRESSES.save(deps.storage, info.sender.clone(), &Empty {})?;

    let contract_address = contract_address.to_string();
    let regsiter_msg = MsgRegisterAsGranter {
        granting_contract: contract_address.clone(),
    };
    let register_stargate_msg = CosmosMsg::Stargate {
        type_url: "/archway.cwfees.v1.Msgenv.contract.addressRegisterAsGranter".to_string(),
        value: Binary::from(prost::Message::encode_to_vec(&regsiter_msg)),
    };
    

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string())
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
        ExecuteMsg::Increment {} => execute::increment(deps),
        ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
        ExecuteMsg::RemoveAllowance(addr) => {
            ALLOWED_ADDRESSES.remove(deps.storage, deps.api.addr_validate(&addr)?);
            Ok(Response::default())
        }
        ExecuteMsg::AddAllowance(addr) => {
            ALLOWED_ADDRESSES.save(deps.storage, deps.api.addr_validate(&addr)?, &Empty::default())?;
            Ok(Response::default())
        }
        ExecuteMsg::SetNFTContract { addr, token_uri } => execute::set_nft_address(deps, info, env, addr, token_uri),
        ExecuteMsg::Register { id } => execute::register(deps, id)
    }
}

pub mod execute {
    use crate::state::{Player, Pokemon, TOKEN};

    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }

    pub fn set_nft_address(deps: DepsMut, info: MessageInfo, env: Env, addr: String, token_uri: String) -> Result<Response, ContractError> {
        let owner = OWNER.load(deps.storage)?;
        if info.sender.to_string() == owner.to_string() {
            let mint: Cw721ExecuteMsg<(), Empty> = Cw721ExecuteMsg::Mint { 
                token_id: 0.to_string(), 
                owner: env.contract.address.to_string(), 
                token_uri: Some(token_uri), 
                extension: () 
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
            return Err(ContractError::Unauthorized {  })
        }
    }

    pub fn register(deps: DepsMut, id: String) -> Result<Response, ContractError> {
        if PLAYERS.has(deps.storage, id.clone()) {
            return Err(ContractError::Unauthorized {  })
        }
        let mut pokemon: Vec<Pokemon> = Vec::new();
        pokemon.push(Pokemon {
            token_id: 0,
            index: 0,
            health: 100
        });

        let player_data = Player {
            id: id.clone(),
            potions: 0,
            berries: 0,
            default_pokemon: 0,
            pokemons: pokemon
        };
        PLAYERS.save(deps.storage, id.clone(), &player_data)?;
        Ok(Response::default())
    }



    pub fn catch_pokemon(deps: DepsMut, info: MessageInfo, env: Env, id: String, token_uri: String) -> Result<Response, ContractError> {
        if PLAYERS.has(deps.storage, id.clone()) {
            return Err(ContractError::Unauthorized {  })
        }
        
        let token = TOKEN.load(deps.storage)?;
        let nft_address = NFT_CONTRACT.load(deps.storage)?;

        let mint: Cw721ExecuteMsg<(), Empty> = Cw721ExecuteMsg::Mint { 
            token_id: (token+1).to_string(), 
            owner: info.sender.clone().to_string(), 
            token_uri: Some(token_uri), 
            extension: () 
        };

        let wasm_msg = WasmMsg::Execute {
            contract_addr: nft_address.clone().to_string(),
            msg: to_json_binary(&mint)?,
            funds: Vec::new(),
        };

        let mut player = PLAYERS.load(deps.storage, id.clone())?;
        player.pokemons.push(Pokemon {
            token_id: token+1,
            index: (player.pokemons.len() as i32)-1,
            health: 100
        });

        PLAYERS.save(deps.storage, id.clone(), &player)?;
        Ok(Response::new().add_message(wasm_msg))

    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
        QueryMsg::CheckAllowance { addr } => to_json_binary(&query::check_allowance(deps, addr)?),
        QueryMsg::GetPlayer { id } => to_json_binary(&query::get_player(deps, id)?)
    }
}

pub mod query {
    use crate::state::Player;

    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
    pub fn check_allowance(deps: Deps, addr: String) -> StdResult<bool> {
        Ok(ALLOWED_ADDRESSES.has(deps.storage, deps.api.addr_validate(&addr)?))
    }
    pub fn get_player(deps: Deps, id: String) -> StdResult<Player> {
        let player = PLAYERS.load(deps.storage, id)?;
        Ok(player)
    }
}

#[entry_point]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    return match msg {
        SudoMsg::CwGrant(grant) => process_grant(deps, grant),
        _ => Err(ContractError::Unauthorized {})
    }
}

fn process_grant(deps: DepsMut, grant: CwGrant) -> Result<Response, ContractError> {
    const TYPE_URL: &str = "/cosmwasm.wasm.v1.MsgExecuteContract";

    for msg in grant.msgs {
        // we check if all the senders are in the allow list
        let addr = deps.api.addr_validate(&msg.sender)?;
        if !ALLOWED_ADDRESSES.has(deps.storage, addr) {
            return Err(ContractError::Unauthorized {})
        }

        // we check the message type url
        if msg.type_url != TYPE_URL {
            return Err(ContractError::DisallowedMessage(msg.type_url))
        }
    }

    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_json};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg { count: 17 };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
