use doge_light_client::{
    common_types::{QHash160, QHash256},
    hash::sha256_impl::{
        hash_impl_sha256_hash_four_buffers_concat, hash_impl_sha256_hash_three_buffers_concat, hash_impl_sha256_two_to_one_bytes
    },
};

use crate::{
    claim::{
        auto_claim_deposits_tree::{
            constants::AUTO_CLAIM_DEPOSITS_TREE_HEIGHT,
            pending_mints_buffer_builder::PendingMintsGroupsBuilder,
        },
        block_tx_output_tree::{
            TXO_BLOCK_FULL_MERKLE_TREE_HEIGHT, TXO_EMPTY_BLOCK_MERKLE_TREE_ROOT,
            TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH, TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH,
            TXO_TREE_INDEX_BITS_TX_NUM_LENGTH,
        },
        transition::validator::block_witness::PsyBridgeClaimBlockWitnessVerifyResult,
    },
    tx_template::get_bridge_deposit_output_script,
    utils::{
        append_only_merkle_tree::AppendOnlyMerkleTreeFixed, bit_buffer::TxBitBufferBuilder,
        sha256_zero_hashes::SHA256_ZERO_HASHES,
    },
};
pub fn hash_deposit_leaf(
    tx_hash: &QHash256,
    output_index: u32,
    depositor_public_key: &QHash256,
    amount: &u64,
) -> QHash256 {
    hash_impl_sha256_hash_four_buffers_concat(
        tx_hash,
        depositor_public_key,
        &output_index.to_le_bytes(),
        &amount.to_le_bytes(),
    )
}
pub fn calcuate_fee(
    total_deposit_amount: u64,
    flat_fee_per_deposit_sats: u64,
    deposit_fee_rate_numerator: u64,
    deposit_fee_rate_denominator: u64,
) -> anyhow::Result<(u64, u64)> {
    let deposit_fee_rate = (deposit_fee_rate_numerator as f64)
        / (deposit_fee_rate_denominator as f64);
    let fees_generated = (total_deposit_amount as f64 * deposit_fee_rate).floor() as u64 + flat_fee_per_deposit_sats;
        
    Ok((fees_generated, total_deposit_amount.checked_sub(fees_generated).ok_or_else(|| anyhow::anyhow!("Fee calculation underflow"))?))
}
pub struct BlockTransitionBuilder {
    pub pending_mints: PendingMintsGroupsBuilder,
    pub deposits_tree: AppendOnlyMerkleTreeFixed<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>,
    pub txo_claimed_txs_in_block_tree: AppendOnlyMerkleTreeFixed<
        TXO_TREE_INDEX_BITS_TX_NUM_LENGTH,
        TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH,
    >,
    pub txo_bit_buffer_builder: TxBitBufferBuilder,
    pub depositor_public_keys: Vec<QHash256>,
    pub depositor_output_scripts: Vec<[u8; 23]>,
    pub total_mints: u32,
    pub total_deposit_amount: u64,
    pub flat_fee_per_deposit_sats: u64,
    pub deposit_fee_rate_numerator: u64,
    pub deposit_fee_rate_denominator: u64,
    pub total_fees_collected: u64,
}

// Append Only Merkle Tree Builder for Auto Claim Deposits Tree
impl BlockTransitionBuilder {
    pub fn new_from_empty_with_total_outputs_hint(
        total_outputs_hint: usize,
        flat_fee_per_deposit_sats: u64,
        deposit_fee_rate_numerator: u64,
        deposit_fee_rate_denominator: u64,
    ) -> Self {
        Self {
            pending_mints: PendingMintsGroupsBuilder::new_with_hint(total_outputs_hint),
            deposits_tree:
                AppendOnlyMerkleTreeFixed::<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>::new_from_empty(),
            txo_claimed_txs_in_block_tree: AppendOnlyMerkleTreeFixed::<
                TXO_TREE_INDEX_BITS_TX_NUM_LENGTH,
                TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH,
            >::new_from_empty(),
            depositor_public_keys: Vec::new(),
            depositor_output_scripts: Vec::new(),
            txo_bit_buffer_builder: TxBitBufferBuilder::new_with_total_outputs_hint(
                total_outputs_hint,
            ),
            total_mints: 0,
            total_deposit_amount: 0,
            flat_fee_per_deposit_sats,
            deposit_fee_rate_numerator,
            deposit_fee_rate_denominator,
            total_fees_collected: 0,
            
        }
    }

    pub fn new_from_siblings(
        total_outputs_hint: usize,
        flat_fee_per_deposit_sats: u64,
        deposit_fee_rate_numerator: u64,
        deposit_fee_rate_denominator: u64,
        claim_deposits_last_siblings: &[QHash256],
        claim_deposits_last_index: u32,
        claim_deposits_last_value: &QHash256,
        bridge_public_key_hash: &QHash160,
        depositor_public_keys: Vec<QHash256>,
    ) -> anyhow::Result<Self> {
        let depositor_output_scripts = depositor_public_keys
            .iter()
            .map(|pk| get_bridge_deposit_output_script(pk, bridge_public_key_hash))
            .collect();
        Ok(Self {
            pending_mints: PendingMintsGroupsBuilder::new_with_hint(total_outputs_hint),
            deposits_tree:
                AppendOnlyMerkleTreeFixed::<AUTO_CLAIM_DEPOSITS_TREE_HEIGHT, 0>::new_from_siblings(
                    claim_deposits_last_siblings,
                    claim_deposits_last_index,
                    claim_deposits_last_value,
                )?,
            txo_claimed_txs_in_block_tree: AppendOnlyMerkleTreeFixed::<
                TXO_TREE_INDEX_BITS_TX_NUM_LENGTH,
                TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH,
            >::new_from_empty(),
            depositor_public_keys,
            depositor_output_scripts,
            txo_bit_buffer_builder: TxBitBufferBuilder::new_with_total_outputs_hint(
                total_outputs_hint,
            ),
            total_mints: 0,
            total_deposit_amount: 0,
            flat_fee_per_deposit_sats,
            deposit_fee_rate_numerator,
            deposit_fee_rate_denominator,
            total_fees_collected: 0,
        })
    }

    pub fn add_deposit(
        &mut self,
        tx_hash: &QHash256,
        output_index: u32,
        public_key_index: usize,
        amount: u64,
    ) -> anyhow::Result<()> {
        if public_key_index >= self.depositor_public_keys.len() {
            anyhow::bail!("Depositor public key index out of bounds");
        }
        let (fee, net_amount) = calcuate_fee(
            amount,
            self.flat_fee_per_deposit_sats,
            self.deposit_fee_rate_numerator,
            self.deposit_fee_rate_denominator,
        )?;
        self.pending_mints
            .append_pending_mint(&self.depositor_public_keys[public_key_index], amount);
        let leaf_hash = hash_deposit_leaf(
            tx_hash,
            output_index,
            &self.depositor_public_keys[public_key_index],
            &net_amount,
        );
        self.deposits_tree.append_leaf(&leaf_hash);
        self.total_mints += 1;
        self.total_fees_collected = self
            .total_fees_collected
            .checked_add(fee)
            .ok_or_else(|| anyhow::anyhow!("Total fees collected overflow"))?;
        self.total_deposit_amount = self
            .total_deposit_amount
            .checked_add(net_amount)
            .ok_or_else(|| anyhow::anyhow!("Total deposit amount overflow"))?;
        Ok(())
    }
    pub fn set_txo_transaction_leaf(&mut self, tx_index_in_block: u32, txo_root_for_tx: &QHash256) {
        self.txo_claimed_txs_in_block_tree
            .skip_to_index_efficient(tx_index_in_block);
        self.txo_claimed_txs_in_block_tree
            .append_leaf(txo_root_for_tx);
    }
    pub fn get_expected_output_script_by_public_key_index(
        &self,
        index: usize,
    ) -> anyhow::Result<&[u8; 23]> {
        if index >= self.depositor_public_keys.len() {
            anyhow::bail!("Depositor public key index out of bounds");
        }
        Ok(&self.depositor_output_scripts[index])
    }
    pub fn finalize(
        &self,
        block_height: u32,
        claim_deposits_last_index: u32,
        claim_deposits_last_value: &QHash256,
        flat_fee_per_deposit_sats: u64,
        deposit_fee_rate_numerator: u64,
        deposit_fee_rate_denominator: u64,
        txo_tree_siblings: &[QHash256],
    ) -> anyhow::Result<PsyBridgeClaimBlockWitnessVerifyResult> {
        let mut index = block_height;
        let mut old_block_tree_root = TXO_EMPTY_BLOCK_MERKLE_TREE_ROOT;
        let mut new_block_tree_root = self.txo_claimed_txs_in_block_tree.current_root;
        for i in 0..TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH {
            if index & 1 == 0 {
                old_block_tree_root = hash_impl_sha256_two_to_one_bytes(
                    &old_block_tree_root,
                    &SHA256_ZERO_HASHES[TXO_BLOCK_FULL_MERKLE_TREE_HEIGHT + i],
                );
                new_block_tree_root = hash_impl_sha256_two_to_one_bytes(
                    &new_block_tree_root,
                    &SHA256_ZERO_HASHES[TXO_BLOCK_FULL_MERKLE_TREE_HEIGHT + i],
                );
            } else {
                let sibling = &txo_tree_siblings[i as usize];
                old_block_tree_root =
                    hash_impl_sha256_two_to_one_bytes(sibling, &old_block_tree_root);
                new_block_tree_root =
                    hash_impl_sha256_two_to_one_bytes(sibling, &new_block_tree_root);
            }
            index >>= 1;
        }
        if index != 0 {
            anyhow::bail!("Block height index too large for TXO tree");
        }

        let old_claimed_txo_tree_root = old_block_tree_root;
        let new_claimed_txo_tree_root = new_block_tree_root;
        let old_auto_claimed_deposits_tree_root = self.deposits_tree.start_root;
        let new_auto_claimed_deposits_tree_root = self.deposits_tree.current_root;
        let start_auto_claimed_deposits_index = if claim_deposits_last_index == 0
            && *claim_deposits_last_value == SHA256_ZERO_HASHES[0]
        {
            0
        } else {
            claim_deposits_last_index + 1
        };
        let end_auto_claimed_deposits_index = self.deposits_tree.next_index;
        
        Ok(PsyBridgeClaimBlockWitnessVerifyResult {
            old_claimed_txo_tree_root,
            new_claimed_txo_tree_root,
            old_auto_claimed_deposits_tree_root,
            new_auto_claimed_deposits_tree_root,
            start_auto_claimed_deposits_index,
            end_auto_claimed_deposits_index,
            fees_collected: self.total_fees_collected,
        })
    }
}
