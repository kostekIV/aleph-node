// TEMP allow as everything gets plugged into each other.
// TODO: Remove before we do a release to ensure there is no hanging code.
#![allow(dead_code)]
#![allow(clippy::type_complexity)]
use sp_core::traits::BareCryptoStorePtr;
use std::sync::Arc;

use futures::Future;

use codec::{Decode, Encode};
use rush::{nodes::NodeIndex, HashT, Unit};
use sc_service::SpawnTaskHandle;

use sp_consensus::BlockImport;

use sc_client_api::{
    backend::{AuxStore, Backend},
    BlockchainEvents, ExecutorProvider, Finalizer, LockImportRun, TransactionFor,
};
use sp_api::ProvideRuntimeApi;
use sp_runtime::traits::Block;

pub(crate) mod communication;
pub mod config;
pub(crate) mod environment;
pub mod hash;
mod party;

mod key_types {
    use sp_runtime::KeyTypeId;

    pub const ALEPH: KeyTypeId = KeyTypeId(*b"alph");
}

mod app {
    use crate::key_types::ALEPH;
    use sp_application_crypto::{app_crypto, ed25519};
    app_crypto!(ed25519, ALEPH);
}

pub type AuthorityId = app::Public;

pub type AuthoritySignature = app::Signature;

pub type AuthorityPair = app::Pair;

#[derive(Clone, Debug, Default, Eq, Hash, Encode, Decode, PartialEq)]
pub struct NodeId {
    auth: AuthorityId,
    index: NodeIndex,
}

impl rush::MyIndex for NodeId {
    fn my_index(&self) -> Option<NodeIndex> {
        unimplemented!()
    }
}

/// Ties an authority identification and a cryptography keystore together for use in
/// signing that requires an authority.
pub struct AuthorityCryptoStore {
    authority_id: AuthorityId,
    crypto_store: BareCryptoStorePtr,
}

impl AuthorityCryptoStore {
    /// Constructs a new authority cryptography keystore.
    pub fn new(authority_id: AuthorityId, crypto_store: BareCryptoStorePtr) -> Self {
        AuthorityCryptoStore {
            authority_id,
            crypto_store,
        }
    }

    /// Returns a references to the authority id.
    pub fn authority_id(&self) -> &AuthorityId {
        &self.authority_id
    }

    /// Returns a reference to the cryptography keystore.
    pub fn crypto_store(&self) -> &BareCryptoStorePtr {
        &self.crypto_store
    }
}

impl AsRef<BareCryptoStorePtr> for AuthorityCryptoStore {
    fn as_ref(&self) -> &BareCryptoStorePtr {
        self.crypto_store()
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Encode, Decode)]
pub struct UnitCoord {
    pub creator: NodeIndex,
    pub round: u64,
}

impl<B: HashT, H: HashT> From<Unit<B, H>> for UnitCoord {
    fn from(unit: Unit<B, H>) -> Self {
        UnitCoord {
            creator: unit.creator(),
            round: unit.round() as u64,
        }
    }
}

impl<B: HashT, H: HashT> From<&Unit<B, H>> for UnitCoord {
    fn from(unit: &Unit<B, H>) -> Self {
        UnitCoord {
            creator: unit.creator(),
            round: unit.round() as u64,
        }
    }
}

pub trait ClientForAleph<B, BE>:
    LockImportRun<B, BE>
    + Finalizer<B, BE>
    + AuxStore
    + BlockchainEvents<B>
    + ProvideRuntimeApi<B>
    + ExecutorProvider<B>
    + BlockImport<B, Transaction = TransactionFor<BE, B>, Error = sp_consensus::Error>
where
    BE: Backend<B>,
    B: Block,
{
}

impl<B, BE, T> ClientForAleph<B, BE> for T
where
    BE: Backend<B>,
    B: Block,
    T: LockImportRun<B, BE>
        + Finalizer<B, BE>
        + AuxStore
        + BlockchainEvents<B>
        + ProvideRuntimeApi<B>
        + ExecutorProvider<B>
        + BlockImport<B, Transaction = TransactionFor<BE, B>, Error = sp_consensus::Error>,
{
}

struct SpawnHandle(SpawnTaskHandle);

impl From<SpawnTaskHandle> for SpawnHandle {
    fn from(sth: SpawnTaskHandle) -> Self {
        SpawnHandle(sth)
    }
}

impl rush::SpawnHandle for SpawnHandle {
    fn spawn(&self, name: &'static str, task: impl Future<Output = ()> + Send + 'static) {
        self.0.spawn(name, task)
    }
}

pub struct AlephConfig<N, C> {
    pub network: N,
    pub party_conf: party::Config,
    pub client: Arc<C>,
    pub spawn_handle: SpawnTaskHandle,
}

pub async fn run_aleph_consensus<B: Block, BE, C, N>(config: AlephConfig<N, C>)
where
    BE: Backend<B> + 'static,
    N: Send + Sync + 'static,
    C: ClientForAleph<B, BE> + Send + Sync + 'static,
{
    let AlephConfig {
        network,
        party_conf,
        client,
        spawn_handle,
    } = config;
    let consensus = party::ConsensusParty::new(party_conf, client, network);

    consensus.run(spawn_handle.into()).await
}
