use crate::{common_types::QHash256, hash::{sha256::btc_hash256_bytes, sha256_impl::{hash_impl_btc_hash256_two_to_one_bytes, hash_impl_sha256_bytes, hash_impl_sha256_two_to_one_bytes}}};

pub fn compute_dogecoin_last_transaction_in_block_merkle_root_in_memory(
    value: QHash256,
    siblings: &[u8],
    mut index: u32,
    claimed_total_transaction_count: u32,
    siblings_count: usize,
) -> Option<QHash256> {
    // 1. Sanity Checks
    if claimed_total_transaction_count == 0 { 
        // we cannot have a block with 0 transactions, because of the coinbase
        return None;
    }
    // The proof index must be the LAST index
    if index != claimed_total_transaction_count - 1 {
        return None;
    }
    if siblings.len() < 32 * siblings_count {
        // not enough sibling data, invalid proof
        return None;
    }
    let mut current = value;
    let mut buf = [0u8; 64];
    for i in 0..siblings_count {
        let sibling = &siblings[i * 32..(i + 1) * 32];
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(&sibling);
        } else {
            // sibling is on the left, we need to check for the odd-leaf rule
            // see CVE-2012-2459, https://github.com/bitcoin/bitcoin/blob/9a29b2d331eed5b4cbd6922f63e397b68ff12447/src/consensus/merkle.cpp#L9
            if sibling == &current {
                // if the sibling is the same as the current, then our merkle path is on a duplicate node path
                // meaning that the block has fewer transactions than claimed_total_transaction_count
                return None;
            }
            buf[0..32].copy_from_slice(&sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = btc_hash256_bytes(&buf);
        index >>= 1;
    }
    if index != 0 {
        // wrong number of siblings
        return None;
    }
    Some(current)
}

pub fn compute_dogecoin_block_transaction_merkle_proof_tree_root_in_memory(
    value: QHash256,
    siblings: &[u8],
    mut index: u32,
    siblings_count: usize,
) -> Option<QHash256> {
    if siblings.len() < 32 * siblings_count {
        // not enough sibling data, invalid proof
        return None;
    }
    let mut current = value;
    let mut buf = [0u8; 64];
    for i in 0..siblings_count {
        let sibling = &siblings[i * 32..(i + 1) * 32];
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(&sibling);
        } else {
            // sibling is on the left, we need to check for the odd-leaf rule
            // see CVE-2012-2459, https://github.com/bitcoin/bitcoin/blob/9a29b2d331eed5b4cbd6922f63e397b68ff12447/src/consensus/merkle.cpp#L9
            if sibling == &current {
                // if the sibling is the same as the current, then our merkle path is on a duplicate node path
                // meaning that the block has fewer transactions than claimed_total_transaction_count
                return None;
            }
            buf[0..32].copy_from_slice(&sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = btc_hash256_bytes(&buf);
        index >>= 1;
    }
    if index != 0 {
        // wrong number of siblings
        return None;
    }
    Some(current)
}



pub fn compute_dogecoin_block_transaction_merkle_proof_tree_root_hash256(
    value: QHash256,
    siblings: &[QHash256],
    mut index: u32,
) -> Option<QHash256> {
    let mut current = value;
    for sibling in siblings.iter() {
        if index & 1 == 0 {
            current = hash_impl_btc_hash256_two_to_one_bytes(&current, sibling);
        } else {
            // sibling is on the left, we need to check for the odd-leaf rule
            // see CVE-2012-2459, https://github.com/bitcoin/bitcoin/blob/9a29b2d331eed5b4cbd6922f63e397b68ff12447/src/consensus/merkle.cpp#L9
            if sibling == &current {
                // if the sibling is the same as the current, then our merkle path is on a duplicate node path
                // meaning that the block has fewer transactions than claimed_total_transaction_count
                return None;
            }
            current = hash_impl_btc_hash256_two_to_one_bytes(sibling, &current);
        }
        index >>= 1;
    }
    if index != 0 {
        // wrong number of siblings
        return None;
    }
    Some(current)
}

pub fn compute_btc_hash256_merkle_root_in_memory(
    value: QHash256,
    siblings: &[u8],
    index: u32,
    siblings_count: usize,
) -> QHash256 {
    assert!(siblings.len() <= 32 * siblings_count);

    let mut current = value;
    let mut index = index;
    let mut buf = [0u8; 64];
    for i in 0..siblings_count {
        let sibling = &siblings[i * 32..(i + 1) * 32];
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(&sibling);
        } else {
            buf[0..32].copy_from_slice(&sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = hash_impl_sha256_bytes(&hash_impl_sha256_bytes(&buf));
        index >>= 1;
    }
    assert!(index == 0);
    current
}

pub fn compute_sha256_merkle_root_in_memory(
    value: QHash256,
    siblings: &[u8],
    index: u64,
    siblings_count: usize,
) -> QHash256 {
    assert!(siblings.len() <= 32 * siblings_count);

    let mut current = value;
    let mut index = index;
    let mut buf = [0u8; 64];
    for i in 0..siblings_count {
        let sibling = &siblings[i * 32..(i + 1) * 32];
        if index & 1 == 0 {
            buf[0..32].copy_from_slice(&current);
            buf[32..64].copy_from_slice(&sibling);
        } else {
            buf[0..32].copy_from_slice(&sibling);
            buf[32..64].copy_from_slice(&current);
        }
        current = hash_impl_sha256_bytes(&buf);
        index >>= 1;
    }
    assert!(index == 0);
    current
}
