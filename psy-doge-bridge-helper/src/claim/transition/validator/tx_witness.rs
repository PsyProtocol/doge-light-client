use doge_light_client::{common_types::QHash256, doge::transaction::BTCTransaction, hash::merkle::in_memory::compute_dogecoin_block_transaction_merkle_proof_tree_root_hash256};

use crate::{claim::transition::transition_builder::BlockTransitionBuilder, utils::bit_vector_append_only_tree_builder::BitVectorAppendOnlyMerkleTreeFixed};
use crate::claim::block_tx_output_tree::TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH;


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct PsyBridgeClaimDepositItem {
    pub output_index: u32,
    pub public_key_index: u32,
}


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct PsyBridgeClaimBlockTransactionWitness {
    pub transaction_index: u32,
    pub btc_transaction_tree_siblings: Vec<QHash256>,
    pub deposit_outputs: Vec<PsyBridgeClaimDepositItem>,
    pub transaction: BTCTransaction,
}


impl PsyBridgeClaimBlockTransactionWitness {
    pub fn new(
        transaction_index: u32,
        btc_transaction_tree_siblings: Vec<QHash256>,
        deposit_outputs: Vec<PsyBridgeClaimDepositItem>,
        transaction: BTCTransaction,
    ) -> Self {
        Self {
            transaction_index,
            btc_transaction_tree_siblings,
            deposit_outputs,
            transaction,
        }
    }



    pub fn verify_and_add_to_transition_builder(&self, transition_builder: &mut BlockTransitionBuilder, block_transaction_tree_root: &QHash256) -> anyhow::Result<()> {
        let transaction_hash = self.transaction.get_hash();
        let computed_root = compute_dogecoin_block_transaction_merkle_proof_tree_root_hash256(
            transaction_hash,
            &self.btc_transaction_tree_siblings,
            self.transaction_index,
        );
        if computed_root.is_none() || computed_root.as_ref().unwrap() != block_transaction_tree_root {
            anyhow::bail!("Transaction Merkle Proof verification failed, merkle proof for block transaction tree root does not match");
        }
        let mut txo_builder = BitVectorAppendOnlyMerkleTreeFixed::<TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH>::new_from_empty();

        let mut last_deposit_output_index = None;
        for claim_output in &self.deposit_outputs {
            if let Some(last_index) = last_deposit_output_index {
                if claim_output.output_index <= last_index {
                    anyhow::bail!("Deposit outputs must be in strictly increasing order by output_index");
                }
            }
            last_deposit_output_index = Some(claim_output.output_index);

            if self.transaction.outputs.len() <= claim_output.output_index as usize {
                anyhow::bail!("Deposit output index out of bounds for transaction outputs");
            }

            let output = &self.transaction.outputs[claim_output.output_index as usize];
            let pub_key_index = claim_output.public_key_index as usize;
            let expected_output_script = transition_builder.get_expected_output_script_by_public_key_index(pub_key_index)?;
            if &output.script != expected_output_script {
                anyhow::bail!("Deposit output script does not match expected script for user public key");
            }

            transition_builder.add_deposit(&transaction_hash, claim_output.output_index, pub_key_index, output.value)?;
            txo_builder.set_true_bit_at(claim_output.output_index);


        }
        transition_builder.set_txo_transaction_leaf(self.transaction_index, &txo_builder.finalize_into_root());
        
        Ok(())
    }
}
