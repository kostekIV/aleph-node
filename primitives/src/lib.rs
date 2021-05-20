#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use sp_core::crypto::KeyTypeId;
use sp_runtime::ConsensusEngineId;
use sp_std::vec::Vec;

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"alp0");

// Same as GRANDPA_ENGINE_ID because as of right now substrate sends only
// grandpa justifications over the network.
// TODO: change this once https://github.com/paritytech/substrate/issues/8172 will be resolved.
pub const ALEPH_ENGINE_ID: ConsensusEngineId = *b"FRNK";

mod app {
    use sp_application_crypto::{app_crypto, ed25519};
    app_crypto!(ed25519, crate::KEY_TYPE);
}

pub type AuthorityId = app::Public;
// TODO set same value as in runtime
pub type BlockNumber = u32;

sp_api::decl_runtime_apis! {
    pub trait AlephApi {
        fn authorities() -> Vec<AuthorityId>;
        fn session() -> Session<AuthorityId,BlockNumber>;
    }
}

#[derive(Decode, Encode, PartialEq, Eq, Clone)]
pub enum AuthoritiesLog<Id, Number>
where
    Id: Encode + Decode,
    Number: Encode + Decode,
{
    WillChange {
        session_id: u64,
        when: Number,
        next_authorities: Vec<Id>,
    },
}

#[derive(Decode, Encode, PartialEq, Eq, Clone)]
pub struct Session<Id, Number>
where
    Id: Encode + Decode,
    Number: Encode + Decode,
{
    pub session_id: u64,
    pub start_h: Number,
    pub stop_h: Number,
    pub authorities: Vec<Id>,
}
