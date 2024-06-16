use cw721_base::msg::ExecuteMsg as Cw721ExecuteMsg;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, Empty ,MessageInfo, Response, StdResult, BankMsg};
use cw2::set_contract_version;


use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::cwfees::{SudoMsg, MsgRegisterAsGranter, CwGrant};
use crate::state::{State, STATE, ALLOWED_ADDRESSES, OWNER, NFT_CONTRACT};

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
    let contract_address = env.contract.address;

    OWNER.save(deps.storage, &info.sender)?;

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
        .add_attribute("count", msg.count.to_string())
        .add_attribute("action", "register")
        .add_message(register_stargate_msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
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
        ExecuteMsg::SetNFTContract(addr) => {
            if info.sender.to_string() == addr {
                NFT_CONTRACT.save(deps.storage, &deps.api.addr_validate(&addr)?)?;
                Ok(Response::default())
            } else {
                return Err(ContractError::Unauthorized {  })
            }
        }
    }
}

pub mod execute {
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
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
        QueryMsg::CheckAllowance { addr } => to_json_binary(&query::check_allowance(deps, addr)?)
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
    pub fn check_allowance(deps: Deps, addr: String) -> StdResult<bool> {
        Ok(ALLOWED_ADDRESSES.has(deps.storage, deps.api.addr_validate(&addr)?))
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
