use doge_light_client::common_types::{QHash160, QHash256};

use crate::claim::{
    auto_claim_deposits_tree::constants::AUTO_CLAIM_DEPOSITS_TREE_HEIGHT,
    block_tx_output_tree::TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH,
    transition::{
        transition_builder::BlockTransitionBuilder,
        validator::tx_witness::PsyBridgeClaimBlockTransactionWitness,
    },
};

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
#[cfg_attr(
    feature = "serialize_bytemuck",
    derive(bytemuck::Pod, bytemuck::Zeroable)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct PsyBridgeClaimBlockWitnessHeader {
    pub txo_tree_block_siblings: [QHash256; TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH],
    pub last_auto_claimed_deposits_siblings: [QHash256; AUTO_CLAIM_DEPOSITS_TREE_HEIGHT],
    pub total_outputs_hint: u32,
    pub claim_deposits_last_index: u32,
    pub claim_deposits_last_value: QHash256,
}

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
#[cfg_attr(
    feature = "serialize_bytemuck",
    derive(bytemuck::Pod, bytemuck::Zeroable)
)]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct PsyBridgeClaimBlockWitnessVerifyResult {
    pub old_claimed_txo_tree_root: QHash256,
    pub new_claimed_txo_tree_root: QHash256,
    pub old_auto_claimed_deposits_tree_root: QHash256,
    pub new_auto_claimed_deposits_tree_root: QHash256,
    pub start_auto_claimed_deposits_index: u32,
    pub end_auto_claimed_deposits_index: u32,
    pub fees_collected: u64,
}

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
pub struct PsyBridgeClaimBlockWitness {
    pub header: PsyBridgeClaimBlockWitnessHeader,
    pub deposit_solana_public_keys: Vec<[u8; 32]>,
    pub deposit_transactions: Vec<PsyBridgeClaimBlockTransactionWitness>,
}


impl PsyBridgeClaimBlockWitness {
    pub fn new(
        header: PsyBridgeClaimBlockWitnessHeader,
        deposit_solana_public_keys: Vec<[u8; 32]>,
        deposit_transactions: Vec<PsyBridgeClaimBlockTransactionWitness>,
    ) -> Self {
        Self {
            header,
            deposit_solana_public_keys,
            deposit_transactions,
        }
    }

    pub fn verify_and_get_result(
        self,
        block_height: u32,
        block_transaction_tree_merkle_root: QHash256,
        bridge_public_key_hash: QHash160,
        flat_fee_per_deposit_sats: u64,
        deposit_fee_rate_numerator: u64,
        deposit_fee_rate_denominator: u64,
    ) -> anyhow::Result<PsyBridgeClaimBlockWitnessVerifyResult> {
        let mut transition_builder = BlockTransitionBuilder::new_from_siblings(
            self.header.total_outputs_hint as usize,
            flat_fee_per_deposit_sats,
            deposit_fee_rate_numerator,
            deposit_fee_rate_denominator,
            &self.header.last_auto_claimed_deposits_siblings,
            self.header.claim_deposits_last_index,
            &self.header.claim_deposits_last_value,
            &bridge_public_key_hash,
            self.deposit_solana_public_keys,
        )?;
        for deposit in self.deposit_transactions {
            deposit.verify_and_add_to_transition_builder(
                &mut transition_builder,
                &block_transaction_tree_merkle_root,
            )?;
        }

        transition_builder.finalize(
            block_height,
            self.header.claim_deposits_last_index,
            &self.header.claim_deposits_last_value,
                flat_fee_per_deposit_sats,
                deposit_fee_rate_numerator,
                deposit_fee_rate_denominator,
                &self.header.txo_tree_block_siblings,
        )
    }
}
