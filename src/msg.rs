use crate::state::{Member, Proposal};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub members: Vec<Member>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Propose {
        title: String,
        description: String,
        recipient: Option<Addr>,
        amount: Option<Uint128>,
    },
    Vote {
        proposal_id: u64,
        approve: bool,
    },
    Execute {
        proposal_id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Proposal)]
    GetProposal { proposal_id: u64 },
    #[returns(Vec<Proposal>)]
    ListProposals {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    #[returns(Member)]
    GetMember { address: Addr },
    #[returns(Vec<Member>)]
    ListMembers {
        start_after: Option<Addr>,
        limit: Option<u32>,
    },
}
