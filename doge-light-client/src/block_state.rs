use crate::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_bytes};


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy, Hash, Default)]
#[repr(C)]
pub struct PsyBridgeStateCommitment {
    pub block_hash: QHash256,
    pub block_merkle_tree_root: QHash256,
    pub auto_claimed_txo_tree_root: QHash256,
    pub auto_claimed_deposits_tree_root: QHash256,
    pub auto_claimed_deposits_next_index: u32,
    pub block_height: u32,
}

#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy, Hash, Default)]
#[repr(C)]
pub struct PsyBridgeHeader {
    pub tip_state: PsyBridgeStateCommitment,
    pub finalized_state: PsyBridgeStateCommitment,
    pub bridge_state_hash: QHash256, // hash of QEDDogeChainStateCore, useful for other contracts that want to introspect dogecoin's blocks
    //pub transition_auto_process_mint_hash_stack: QHash256,
    pub last_rollback_at_secs: u32,
    pub paused_until_secs: u32,
    pub total_finalized_fees_collected_chain_history: u64,

    //pub auto_collected_fees_from_finalized_to_tip_stats: u64,
}

impl PsyBridgeHeader {
    pub fn is_paused(&self, current_unix_timestamp_secs: u32) -> bool {
        self.paused_until_secs > current_unix_timestamp_secs
    }

    #[cfg(feature = "serialize_bytemuck")]
    pub fn get_hash_bm(&self) -> QHash256 {
        hash_impl_sha256_bytes(bytemuck::bytes_of(self))
    }

    #[cfg(feature = "serialize_bytemuck")]
    pub fn from_bytes_ref_or_panic_bm(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes::<Self>(bytes)
    }

    #[cfg(feature = "serialize_bytemuck")]
    pub fn to_bytes_ref_bm(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    pub fn to_canonical_bytes_ref(&self) -> &[u8] {
        #[cfg(feature = "serialize_bytemuck")]
        {
            self.to_bytes_ref_bm()
        }
        #[cfg(not(feature = "serialize_bytemuck"))]
        {
            panic!("to_canonical_bytes_ref requires 'serialize_bytemuck' feature");
        }
    }
    pub fn get_hash_canonical(&self) -> QHash256 {
        #[cfg(feature = "serialize_bytemuck")]
        {
            self.get_hash_bm()
        }
        #[cfg(not(feature = "serialize_bytemuck"))]
        {
            panic!("get_hash_canonical requires 'serialize_bytemuck' feature");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bridge_header_hash() {
        let header = PsyBridgeHeader {
            tip_state: PsyBridgeStateCommitment::default(),
            finalized_state: PsyBridgeStateCommitment::default(),
            bridge_state_hash: QHash256::default(),
            last_rollback_at_secs: 0,
            paused_until_secs: 0,
            total_finalized_fees_collected_chain_history: 5000,
            //transition_auto_process_mint_hash_stack: QHash256::default(),
        };
        let hash = header.get_hash_bm();

        println!("Bridge Header Hash: {}", hex::encode(hash));
    }
}
