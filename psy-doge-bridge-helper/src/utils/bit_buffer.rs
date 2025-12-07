use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_bytes};

use crate::claim::block_tx_output_tree::TXO_TREE_MAX_OUTPUTS_PER_TX;

pub struct TxBitBufferBuilder {
    pub outputs: Vec<u32>,
}

impl TxBitBufferBuilder {
    pub fn new() -> Self {
        Self {
            outputs: Vec::new(),
        }
    }
    pub fn new_with_total_outputs_hint(total_outputs_hint: usize) -> Self {
        Self {
            outputs: Vec::with_capacity(total_outputs_hint),
        }
    }
    pub fn append_output(&mut self, transaction_index: u32, output_in_tx_index: u32) {
        self.outputs.push(transaction_index * TXO_TREE_MAX_OUTPUTS_PER_TX as u32 + output_in_tx_index);
    }
    pub fn get_hash(&self) -> QHash256 {
        hash_impl_sha256_bytes(&bytemuck::cast_slice(&self.outputs))
    }
}