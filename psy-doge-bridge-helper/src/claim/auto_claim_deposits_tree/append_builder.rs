use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_hash_three_buffers_concat};

use crate::{claim::auto_claim_deposits_tree::{constants::AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, pending_mints_buffer_builder::PendingMintsGroupsBuilder}, utils::append_only_merkle_tree::AppendOnlyMerkleTreeFixed};

pub struct AutoClaimDepositsTreeAppendBuilder {
    pub pending_mints: PendingMintsGroupsBuilder,
    pub deposits_tree: AppendOnlyMerkleTreeFixed<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>,
}

// Append Only Merkle Tree Builder for Auto Claim Deposits Tree
impl AutoClaimDepositsTreeAppendBuilder {
    pub fn new_from_empty_with_total_outputs_hint(total_outputs_hint: usize) -> Self {
        Self {
            pending_mints: PendingMintsGroupsBuilder::new_with_hint(total_outputs_hint),
            deposits_tree: AppendOnlyMerkleTreeFixed::<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>::new_from_empty(),
        }
    }

    pub fn new_from_siblings(
        total_outputs_hint: usize,
        txo_last_siblings: &[QHash256],
        txo_last_index: u32,
        txo_last_value: &QHash256,
    ) -> anyhow::Result<Self> {
        Ok(
            Self { 
                pending_mints: PendingMintsGroupsBuilder::new_with_hint(total_outputs_hint),    
                deposits_tree: AppendOnlyMerkleTreeFixed::<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>::new_from_siblings(txo_last_siblings, txo_last_index, txo_last_value)?
             }
        )
    }

    pub fn append_leaf(&mut self, tx_hash: &QHash256, public_key: &QHash256, amount: u64) {
        self.pending_mints.append_pending_mint(public_key, amount);
        let leaf_hash = hash_impl_sha256_hash_three_buffers_concat(
            tx_hash,
            public_key,
            &amount.to_le_bytes(),
        );
        self.deposits_tree.append_leaf(&leaf_hash);

    }
}