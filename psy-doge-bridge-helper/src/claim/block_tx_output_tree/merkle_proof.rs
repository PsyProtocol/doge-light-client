use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_bytes};

use crate::{claim::block_tx_output_tree::{TXO_MERKLE_TREE_HEIGHT, is_valid_siblings_length_for_txo_merkle_proof, is_valid_txo_merkle_index}, utils::bit_vector::{get_bit_in_bit_vector, set_bit_in_bit_vector_cloned}};

pub fn compute_txo_merkle_root(
    output_bit_vector_leaf: QHash256,
    merkle_siblings: &[QHash256],
    txo_merkle_index: u64,
) -> QHash256 {
    assert!(is_valid_siblings_length_for_txo_merkle_proof(merkle_siblings.len()));
    assert!(is_valid_txo_merkle_index(txo_merkle_index));

    let mut current = output_bit_vector_leaf;
    let mut index = txo_merkle_index;
    let mut buf = [0u8; 64];
    for sibling in merkle_siblings {
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(sibling);
        } else {
            buf[0..32].copy_from_slice(sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = hash_impl_sha256_bytes(&buf);
        index >>= 1;
    }
    assert!(index == 0);
    
    current
}



pub fn compute_txo_merkle_proof_in_memory(
    output_bit_vector_leaf: QHash256,
    merkle_siblings_buffer: &[u8],
    txo_merkle_index: u64,
) -> QHash256 {
    assert!(merkle_siblings_buffer.len() >= TXO_MERKLE_TREE_HEIGHT * 32);
    assert!(is_valid_txo_merkle_index(txo_merkle_index));

    let mut current = output_bit_vector_leaf;
    let mut index = txo_merkle_index;
    let mut buf = [0u8; 64];
    for i in 0..TXO_MERKLE_TREE_HEIGHT {
        let sibling = &merkle_siblings_buffer[i * 32..(i + 1) * 32];
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(sibling);
        } else {
            buf[0..32].copy_from_slice(sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = hash_impl_sha256_bytes(&buf);
        index >>= 1;
    }
    assert!(index == 0);
    
    current
}



pub fn compute_txo_delta_merkle_root(
    old_output_bit_vector_leaf: QHash256,
    new_output_bit_vector_leaf: QHash256,
    merkle_siblings: &[QHash256],
    txo_merkle_index: u64,
) -> (QHash256, QHash256) {
    assert!(is_valid_siblings_length_for_txo_merkle_proof(merkle_siblings.len()));
    assert!(is_valid_txo_merkle_index(txo_merkle_index));

    let mut current_old = old_output_bit_vector_leaf;
    let mut current_new = new_output_bit_vector_leaf;
    let mut index = txo_merkle_index;
    let mut buf = [0u8; 64];
    for sibling in merkle_siblings {
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current_old);
            buf[32..64].copy_from_slice(sibling);
            current_old = hash_impl_sha256_bytes(&buf);
            buf[0..32].copy_from_slice(&current_new);
            current_new = hash_impl_sha256_bytes(&buf);
        } else {
            buf[0..32].copy_from_slice(sibling);
            buf[32..64].copy_from_slice(&current_old);
            current_old = hash_impl_sha256_bytes(&buf);
            buf[32..64].copy_from_slice(&current_new);
            current_new = hash_impl_sha256_bytes(&buf);
        }
        index >>= 1;
    }
    assert!(index == 0);
    
    (current_old, current_new)
}



pub fn compute_txo_delta_merkle_root_set_zero_bit_to_one(
    old_output_bit_vector_leaf: QHash256,
    bit_index: u8,
    merkle_siblings: &[QHash256],
    txo_merkle_index: u64,
) -> (QHash256, QHash256) {
    assert!(is_valid_siblings_length_for_txo_merkle_proof(merkle_siblings.len()));
    assert!(is_valid_txo_merkle_index(txo_merkle_index));
    assert!(!get_bit_in_bit_vector(&old_output_bit_vector_leaf, bit_index));
    let new_output_bit_vector_leaf = set_bit_in_bit_vector_cloned(&old_output_bit_vector_leaf, bit_index);

    
    let mut current_old = old_output_bit_vector_leaf;
    let mut current_new = new_output_bit_vector_leaf;
    let mut index = txo_merkle_index;
    let mut buf = [0u8; 64];
    for sibling in merkle_siblings {
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current_old);
            buf[32..64].copy_from_slice(sibling);
            current_old = hash_impl_sha256_bytes(&buf);
            buf[0..32].copy_from_slice(&current_new);
            current_new = hash_impl_sha256_bytes(&buf);
        } else {
            buf[0..32].copy_from_slice(sibling);
            buf[32..64].copy_from_slice(&current_old);
            current_old = hash_impl_sha256_bytes(&buf);
            buf[32..64].copy_from_slice(&current_new);
            current_new = hash_impl_sha256_bytes(&buf);
        }
        index >>= 1;
    }
    assert!(index == 0);
    
    (current_old, current_new)
}