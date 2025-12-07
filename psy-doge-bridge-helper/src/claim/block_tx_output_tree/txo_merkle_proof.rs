use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_two_to_one_bytes};

use crate::claim::block_tx_output_tree::{TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH, TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH, TXO_TREE_INDEX_BITS_TX_NUM_LENGTH, get_output_in_tx_merkle_index_bit_index, get_output_in_tx_merkle_index_bit_index_byte_index_bit_mask};


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct TXOOutputInTransactionMerkleProofPartial {
    pub value: QHash256,
    pub siblings: [QHash256; TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH],
    pub index: u32,
}
impl TXOOutputInTransactionMerkleProofPartial {
    pub fn new(
        value: QHash256,
        siblings: [QHash256; TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH],
        index: u32,
    ) -> Self {
        Self {
            value,
            siblings,
            index,
        }
    }
    pub fn compute_root(&self) -> QHash256 {
        let mut current = self.value;
        let mut index = self.index;
        for i in 0..TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH {
            let sibling = self.siblings[i];
            if index & 1 == 0 {
                current = hash_impl_sha256_two_to_one_bytes(&current, &sibling);
            } else {
                current = hash_impl_sha256_two_to_one_bytes(&sibling, &current);
            }
            index >>= 1;
        }
        assert!(index == 0);
        current
    }
}


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct TXOOutputInTransactionDeltaMerkleProofPartial {
    pub old_value: QHash256,
    pub siblings: [QHash256; TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH],
    pub merkle_index: u8,
    pub bit_index_in_leaf: u8,
    pub old_bit: u8,
    pub new_bit: u8,
}
impl TXOOutputInTransactionDeltaMerkleProofPartial {
    pub fn new(
        old_value: QHash256,
        siblings: [QHash256; TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH],
        output_index: u16,
        new_bit: u8,
    ) -> Self {
        let (merkle_index, bit_index, byte_index, bit_mask) = get_output_in_tx_merkle_index_bit_index_byte_index_bit_mask(output_index);
        Self {
            old_bit: old_value[byte_index as usize] >> bit_mask & 1,
            old_value,
            siblings,
            merkle_index,
            bit_index_in_leaf: bit_index,
            new_bit,
        }
    }
    pub fn compute_roots(&self) -> (QHash256, QHash256) {
        let mut current = self.old_value;
        let mut current_new = self.old_value;
        // Apply the bit change
        if self.new_bit == 1 {
            current_new[(self.bit_index_in_leaf >> 3) as usize] |= 1 << (self.bit_index_in_leaf & 7);
        } else {
            current_new[(self.bit_index_in_leaf >> 3) as usize] &= !(1 << (self.bit_index_in_leaf & 7));
        }
        let mut index = self.merkle_index as u32;
        for i in 0..TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH {
            let sibling = self.siblings[i];
            if index & 1 == 0 {
                current = hash_impl_sha256_two_to_one_bytes(&current, &sibling);
                current_new = hash_impl_sha256_two_to_one_bytes(&current_new, &sibling);
            } else {
                current = hash_impl_sha256_two_to_one_bytes(&sibling, &current);
                current_new = hash_impl_sha256_two_to_one_bytes(&sibling, &current_new);
            }
            index >>= 1;
        }
        assert!(index == 0);

        (current, current_new)
    }
}

#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct TXONestedTransactionInBlockMerkleProofPartial {
    pub siblings: [QHash256; TXO_TREE_INDEX_BITS_TX_NUM_LENGTH],
    pub index: u32,
    pub txo_output_proof: TXOOutputInTransactionMerkleProofPartial,
}
impl TXONestedTransactionInBlockMerkleProofPartial {
    pub fn compute_root(&self) -> QHash256 {
        let mut current = self.txo_output_proof.compute_root();
        let mut index = self.index;
        for i in 0..TXO_TREE_INDEX_BITS_TX_NUM_LENGTH {
            let sibling = self.siblings[i];
            if index & 1 == 0 {
                current = hash_impl_sha256_two_to_one_bytes(&current, &sibling);
            } else {
                current = hash_impl_sha256_two_to_one_bytes(&sibling, &current);
            }
            index >>= 1;
        }
        assert!(index == 0);
        current
    }
}



#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
#[repr(C)]
pub struct TXONestedMerkleProofPartial {
    pub siblings: [QHash256; TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH],
    pub index: u32,
    pub tx_in_block_proof: TXONestedTransactionInBlockMerkleProofPartial,
}
impl TXONestedMerkleProofPartial {
    pub fn compute_root(&self) -> QHash256 {
        let mut current = self.tx_in_block_proof.compute_root();
        let mut index = self.index;
        for i in 0..TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH {
            let sibling = self.siblings[i];
            if index & 1 == 0 {
                current = hash_impl_sha256_two_to_one_bytes(&current, &sibling);
            } else {
                current = hash_impl_sha256_two_to_one_bytes(&sibling, &current);
            }
            index >>= 1;
        }
        assert!(index == 0);
        current
    }
}

