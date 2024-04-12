use std::collections::HashSet;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::Map;

#[cw_serde]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub recipient: Addr,
    pub amount: Uint128,
    pub votes_for: Uint128,
    pub votes_against: Uint128,
    pub voters: HashSet<Addr>,
    pub executed: bool,
     // UNIX timestamp indicating when the proposal expires
     pub voting_end: u64, // UNIX timestamp
    

}

#[cw_serde]
pub struct Member {
    pub address: Addr,
    pub weight: Uint128,
}

pub const PROPOSALS: Map<u64, Proposal> = Map::new("proposals");
pub const MEMBERS: Map<Addr, Member> = Map::new("members");