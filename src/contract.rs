use std::collections::HashSet;

use cosmwasm_std::{
    entry_point, to_json_binary, Addr, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdResult, Uint128,
};
use cw2::set_contract_version;
use cw_storage_plus::Bound;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Member, Proposal, MEMBERS, PROPOSALS};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DENOM: &str = "udevcore";

// pagination info for queries
const MAX_PAGE_LIMIT: u32 = 250;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    for member in msg.members {
        MEMBERS.save(
            deps.storage,
            deps.api.addr_validate(member.address.as_str())?,
            &Member {
                address: member.address.clone(),
                weight: member.weight,
            },
        )?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Propose {
            title,
            description,
            recipient,
            amount,
        } => execute_propose(deps, info, title, description, recipient, amount),
        ExecuteMsg::Vote {
            proposal_id,
            approve,
        } => execute_vote(deps, info, proposal_id, approve),
        ExecuteMsg::Execute { proposal_id } => execute_execute(deps, env, proposal_id), // Add env here
    }
}

fn execute_propose(
    deps: DepsMut,
    info: MessageInfo,
    title: String,
    description: String,
    recipient: Option<Addr>,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let member_opt = MEMBERS.load(deps.storage, info.sender.clone());

    if member_opt.is_err() {
        return Err(ContractError::Unauthorized {});
    }

    let proposal = Proposal {
        id: 0,
        title,
        description,
        votes_for: Uint128::zero(),
        votes_against: Uint128::zero(),
        voters: HashSet::new(),
        executed: false,
        amount: amount.unwrap_or_default(),
        recipient: recipient.unwrap_or(info.sender),
    };

    PROPOSALS.save(deps.storage, proposal.id, &proposal)?;

    Ok(Response::default())
}

fn execute_vote(
    deps: DepsMut,
    info: MessageInfo,
    proposal_id: u64,
    approve: bool,
) -> Result<Response, ContractError> {
    let member = MEMBERS
        .load(deps.storage, info.sender.clone())
        .map_err(|_| ContractError::Unauthorized {})?;

    let mut proposal = PROPOSALS
        .load(deps.storage, proposal_id)
        .map_err(|_| ContractError::ProposalDoesNotExist {})?;

    if proposal.voters.contains(&info.sender) {
        return Err(ContractError::MemberAlreadyVoted {});
    }

    if approve {
        proposal.votes_for += member.weight;
    } else {
        proposal.votes_against += member.weight;
    }

    proposal.voters.insert(info.sender);

    PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

    Ok(Response::default())
}

fn execute_execute(deps: DepsMut, env: Env, proposal_id: u64) -> Result<Response, ContractError> {
    let mut proposal = PROPOSALS.load(deps.storage, proposal_id)?;

    if proposal.executed {
        return Err(ContractError::AlreadyExecuted {});
    }

    let mut response = Response::new();

    if proposal.votes_for > proposal.votes_against {
        if deps
            .querier
            .query_balance(env.contract.address, DENOM)?
            .amount
            < proposal.amount
        {
            return Err(ContractError::InsufficientFunds {});
        }

        proposal.executed = true;
        PROPOSALS.save(deps.storage, proposal_id, &proposal)?;

        if !proposal.amount.is_zero() {
            let transfer = BankMsg::Send {
                to_address: proposal.recipient.to_string(),
                amount: vec![Coin {
                    denom: DENOM.to_string(),
                    amount: proposal.amount,
                }],
            };

            response = response.add_message(transfer);
        }

        return Ok(response
            .add_attribute("method", "execute_execute")
            .add_attribute("recipient", proposal.recipient)
            .add_attribute("amount", proposal.amount));
    }

    Ok(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProposal { proposal_id } => {
            to_json_binary(&query_get_proposal(deps, proposal_id)?)
        }
        QueryMsg::ListProposals { start_after, limit } => {
            to_json_binary(&query_list_proposals(deps, start_after, limit))
        }
        QueryMsg::GetMember { address } => to_json_binary(&query_get_member(deps, address)?),
        QueryMsg::ListMembers { start_after, limit } => {
            to_json_binary(&query_list_members(deps, start_after, limit))
        }
    }
}

fn query_get_proposal(deps: Deps, proposal_id: u64) -> StdResult<Proposal> {
    let proposal = PROPOSALS.load(deps.storage, proposal_id)?;
    Ok(proposal)
}

fn query_get_member(deps: Deps, address: Addr) -> StdResult<Member> {
    let member = MEMBERS.load(deps.storage, address)?;
    Ok(member)
}

fn query_list_proposals(deps: Deps, start_after: Option<u64>, limit: Option<u32>) -> Vec<Proposal> {
    let limit = limit.unwrap_or(MAX_PAGE_LIMIT).min(MAX_PAGE_LIMIT);
    let start = start_after.map(Bound::exclusive);

    let proposals = PROPOSALS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit as usize)
        .filter_map(Result::ok)
        .map(|(_, proposal)| proposal)
        .collect();

    proposals
}

fn query_list_members(deps: Deps, start_after: Option<Addr>, limit: Option<u32>) -> Vec<Member> {
    let limit = limit.unwrap_or(MAX_PAGE_LIMIT).min(MAX_PAGE_LIMIT);
    let start = start_after.map(Bound::exclusive);

    let members = MEMBERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit as usize)
        .filter_map(Result::ok)
        .map(|(_, member)| member)
        .collect();

    members
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Member;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, Addr, Empty, Uint128};
    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    fn dao_contract() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(execute, instantiate, query);
        Box::new(contract)
    }

    #[test]
    fn proper_instantiation() {
        let mut deps = mock_dependencies();

        let members = vec![
            Member {
                address: Addr::unchecked("addr1"),
                weight: Uint128::from(10_u128),
            },
            Member {
                address: Addr::unchecked("addr2"),
                weight: Uint128::from(20_u128),
            },
        ];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn proposal_creation() {
        let mut deps = mock_dependencies();

        let members = vec![Member {
            address: Addr::unchecked("addr1"),
            weight: Uint128::from(10_u128),
        }];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Propose
        let info = mock_info("addr1", &[]);
        let msg = ExecuteMsg::Propose {
            title: "Test Proposal".to_string(),
            description: "Description for test".to_string(),
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn vote_for_proposal() {
        let mut deps = mock_dependencies();

        let members = vec![Member {
            address: Addr::unchecked("addr1"),
            weight: Uint128::from(10_u128),
        }];
        let msg = InstantiateMsg { members };
        let info = mock_info("creator", &[]);
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Propose
        let info = mock_info("addr1", &[]);
        let proposal_msg = ExecuteMsg::Propose {
            title: "Some Title".to_string(),
            description: "Some Description".to_string(),
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), proposal_msg).unwrap();

        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 0,
            approve: true,
        };

        let res = execute(deps.as_mut(), mock_env(), info.clone(), vote_msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        // Voting again should fail
        execute(deps.as_mut(), mock_env(), info, vote_msg).unwrap_err();
    }

    #[test]
    fn execute_proposal() {
        let sender = Addr::unchecked("sender");
        let mut app = App::new(|router, _api, storage| {
            router
                .bank
                .init_balance(storage, &sender, coins(100_000_000_000, DENOM))
                .unwrap();
        });

        let contract_id = app.store_code(dao_contract());

        let members = vec![Member {
            address: sender.clone(),
            weight: Uint128::from(10_u128),
        }];

        let contract_addr = app
            .instantiate_contract(
                contract_id,
                sender.clone(),
                &InstantiateMsg { members },
                &[],
                "grant-dao",
                None,
            )
            .unwrap();

        // Propose
        let proposal_msg = ExecuteMsg::Propose {
            title: "Some Title".to_string(),
            description: "Some Description".to_string(),
            amount: Some(Uint128::from(100_u128)),
            recipient: Some(Addr::unchecked("recipient_address")),
        };

        app.execute_contract(sender.clone(), contract_addr.clone(), &proposal_msg, &[])
            .unwrap();

        // Vote
        let vote_msg = ExecuteMsg::Vote {
            proposal_id: 0,
            approve: true,
        };

        app.execute_contract(sender.clone(), contract_addr.clone(), &vote_msg, &[])
            .unwrap();

        // Execute the proposal
        // Should fail because no funds in contract
        let execute_msg = ExecuteMsg::Execute { proposal_id: 0 };
        app.execute_contract(sender.clone(), contract_addr.clone(), &execute_msg, &[])
            .unwrap_err();

        // Send funds to contract so that proposal can be executed
        app.send_tokens(sender.clone(), contract_addr.clone(), &coins(100, DENOM))
            .unwrap();

        // Executing the proposal should succeed now
        app.execute_contract(sender.clone(), contract_addr.clone(), &execute_msg, &[])
            .unwrap();

        // Check balance of recipient
        let balance = app
            .wrap()
            .query_balance(Addr::unchecked("recipient_address"), DENOM)
            .unwrap();

        assert_eq!(balance, coin(100, DENOM));
    }
}
