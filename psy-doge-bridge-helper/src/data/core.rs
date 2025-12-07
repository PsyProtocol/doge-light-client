use doge_light_client::{block_state::PsyBridgeHeader, chain_state::QEDDogeChainStateCore, common_types::QHash256, core_data::QDogeBlockHeader};
use speedy::{Readable, Writable};
use zerocopy::{FromBytes, IntoBytes};

use crate::{claim::transition::validator::block_witness::PsyBridgeClaimBlockWitness, constants::{PSY_DOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, PSY_DOGE_BRIDGE_BLOCK_TREE_HEIGHT}};

pub type PsyDogeBridgeState = QEDDogeChainStateCore<
    PSY_DOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
    PSY_DOGE_BRIDGE_BLOCK_TREE_HEIGHT,
>;

#[cfg_attr(
    feature = "serialize_serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[cfg_attr(
    feature = "serialize_borsh",
    derive(borsh::BorshSerialize, borsh::BorshDeserialize)
)]
#[cfg_attr(
    feature = "serialize_speedy",
    derive(speedy::Readable, speedy::Writable)
)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct PsyDogeBridgeIncomingBlockWitness {
    pub block_header: QDogeBlockHeader,
    pub claim_witness: PsyBridgeClaimBlockWitness,
    pub previous_header_last_rollback_at_secs: u32,
    pub previous_header_paused_until_secs: u32,
}




#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct PsyDogeBridgeIncomingBlockWitnessAndState {
    pub witness: PsyDogeBridgeIncomingBlockWitness,
    pub state: PsyDogeBridgeState,
}

impl PsyDogeBridgeIncomingBlockWitnessAndState {
    pub fn new(
        witness: PsyDogeBridgeIncomingBlockWitness,
        state: PsyDogeBridgeState,
    ) -> Self {
        Self { witness, state }
    }
    #[cfg(any(feature = "serialize_speedy", test))]
    pub fn to_serialize_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let state_bytes = self.state.as_bytes();
        let mut bytes = Vec::with_capacity(state_bytes.len() + 4096);
        bytes.extend_from_slice(state_bytes.len().to_le_bytes().as_ref());
        bytes.extend_from_slice(state_bytes);
        self.witness.write_to_buffer(&mut bytes)?;
        Ok(bytes)
    }
}

