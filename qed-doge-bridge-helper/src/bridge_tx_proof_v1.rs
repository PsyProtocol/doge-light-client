use doge_light_client::{
    chain_state::QEDDogeChainStateCore, core_data::QHash256, doge::transaction::BTCTransaction, hash::sha256_impl::hash_impl_sha256_bytes
};

use crate::{
    error::{ClaimDogeBridgeHelperError, QClaimDogeResult},
    tx_template::is_bridge_desposit_output_v1_for_user,
};

//fn verify_merkle_proof_in_mem
const MIN_POSSIBLE_TX_SIZE: usize = 60;
const MAX_REASONABLE_TX_SIZE: usize = 1024 * 1024 * 10;

fn compute_merkle_in_mem_btc_has256(
    value: [u8; 32],
    siblings: &[u8],
    index: u32,
    siblings_count: usize,
) -> [u8; 32] {
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

fn compute_merkle_in_mem_sha256(
    value: [u8; 32],
    siblings: &[u8],
    index: u64,
    siblings_count: usize,
) -> [u8; 32] {
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


pub const fn get_user_claimed_combined_index(
    block_number: u32,
    transaction_index: u32,
    output_index: u32,
) -> (u64, u8) {
    assert!(output_index <= 0xFFFFF);
    assert!(transaction_index <= 0xFFFFF);

    let claimed_bit_vector_index = output_index as u8;
    let combined_merkle_index = (block_number as u64) << 32
        | (transaction_index as u64) << 12
        | ((output_index as u64) >> 8);
    (combined_merkle_index, claimed_bit_vector_index)
}

pub fn with_set_bit_in_bit_vector(bit_vector: &[u8; 32], bit: u8) -> [u8; 32] {
    let byte_index = bit >> 3; // divide by 8, get the byte index
    let bit_index = bit & 7; // mod 8, get the bit index

    let mut new_bit_vector = *bit_vector;
    new_bit_vector[byte_index as usize] |= 1 << bit_index;
    new_bit_vector
}

pub const fn get_claimed_bit_status(claimed_bit_vector: &[u8], index: u8) -> bool {
    let byte_index = index >> 3; // divide by 8, get the byte index
    let bit_index = index & 7; // mod 8, get the bit index

    (claimed_bit_vector[byte_index as usize] & (1 << bit_index)) != 0
}

pub struct UserClaimStateProofV1 {
    pub tx_in_block_proof: TransactionInBlockProofV1,
    pub old_claimed_bit_vector: QHash256,
    pub claim_tree_siblings: [QHash256; 64],
}

impl UserClaimStateProofV1 {
    pub fn new(
        tx_in_block_proof: TransactionInBlockProofV1,
        old_claimed_bit_vector: QHash256,
        claim_tree_siblings: [QHash256; 64],
    ) -> Self {
        UserClaimStateProofV1 {
            tx_in_block_proof,
            old_claimed_bit_vector,
            claim_tree_siblings,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let tx_in_block_proof_bytes = self.tx_in_block_proof.to_bytes();

        let size = tx_in_block_proof_bytes.len() + 32 + 32 * 64;

        let mut bytes = Vec::with_capacity(size);
        bytes.extend_from_slice(&tx_in_block_proof_bytes);
        bytes.extend_from_slice(&self.old_claimed_bit_vector);
        for sibling in self.claim_tree_siblings.iter() {
            bytes.extend_from_slice(sibling);
        }
        assert!(bytes.len() == size);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<(Self, usize)> {
        let (tx_in_block_proof, offset) = TransactionInBlockProofV1::from_bytes(&bytes[0..])?;
        let mut offset = offset;
        let mut old_claimed_bit_vector = [0u8; 32];
        old_claimed_bit_vector.copy_from_slice(&bytes[offset..offset + 32]);
        offset += 32;
        let mut claim_tree_siblings = [QHash256::default(); 64];
        for i in 0..64 {
            claim_tree_siblings[i].copy_from_slice(&bytes[offset..offset + 32]);
            offset += 32;
        }
        Ok((UserClaimStateProofV1 {
            tx_in_block_proof,
            old_claimed_bit_vector,
            claim_tree_siblings,
        }, offset))
    }

    pub fn verify_tx_out_in_block_is_deposit_v1_with_ibc<'a, 
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize,
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,>(
        solana_public_key: &[u8; 32],
        bridge_public_key_hash: &[u8; 20],
        block_number: u32,
        tx_index: u32,
        output_index: u32,
        ibc: &QEDDogeChainStateCore<QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>,
        known_user_claim_merkle_hash: &[u8; 32],
        data: &'a [u8],
    ) -> QClaimDogeResult<(QHash256, u64)> {
        if ibc.block_data_tracker.get_finalized_block_number() < block_number {
            return Err(ClaimDogeBridgeHelperError::BlockNotFinalized);
        }

        let known_block_tx_merkle_root = ibc.block_data_tracker.get_record(block_number).map_err(|_| {
            ClaimDogeBridgeHelperError::BlockNotInCache
        })?.tx_tree_merkle_root;

        let (new_user_claim_merkle_hash, amount) = Self::verify_tx_out_in_block_is_deposit_v1(
            solana_public_key,
            bridge_public_key_hash,
            block_number,
            tx_index,
            output_index,
            &known_block_tx_merkle_root,
            known_user_claim_merkle_hash,
            data,
        )?;
        Ok((new_user_claim_merkle_hash, amount))
        
    }

    pub fn verify_tx_out_in_block_is_deposit_v1<'a>(
        solana_public_key: &[u8; 32],
        bridge_public_key_hash: &[u8; 20],
        block_number: u32,
        tx_index: u32,
        output_index: u32,
        known_block_tx_merkle_root: &[u8; 32],
        known_user_claim_merkle_hash: &[u8; 32],
        data: &'a [u8],
    ) -> QClaimDogeResult<(QHash256, u64)> {
        let (_, tx_bytes, read_length) =
            TransactionInBlockProofV1::get_proof_tx_in_block(data, tx_index, known_block_tx_merkle_root)?;
        let amount = TransactionInBlockProofV1::check_is_deposit_address_v1_mem(
            tx_bytes,
            output_index as usize,
            solana_public_key,
            bridge_public_key_hash,
        )?;

        // we now know there exists a transaction with an output (block_number, tx_index, output_index) in a block with the known block merkle root of known_block_tx_merkle_root
        // we also know it is a valid deposit for a bridge with the given <bridge_public_key_hash> and the user with address <solana_public_key> of amount <amount> DOGE
        // BUT: we still need to make sure it hasn't been claimed by the user before, see code below

        // TODO: change this to a sparse merkle proof or similar construction to reduce the size of the proof

        let (combined_merkle_index, claimed_bit_vector_index) =
            get_user_claimed_combined_index(block_number, tx_index, output_index);

        let claimed_bit_vector_byte_index = (claimed_bit_vector_index >> 3) as usize; // divide by 8, get the byte index
        let claimed_bit_vector_bit_mask = 1u8 << (claimed_bit_vector_index & 7); // 1<<(i mod 8, get the bit index)

        let mut offset = read_length;
        let mut old_claimed_bit_vector = [0u8; 32];
        old_claimed_bit_vector.copy_from_slice(&data[offset..offset + 32]);

        offset += 32;

        if old_claimed_bit_vector[claimed_bit_vector_byte_index] & claimed_bit_vector_bit_mask != 0
        {
            return Err(ClaimDogeBridgeHelperError::BridgeTransactionAlreadyClaimed);
        }

        let old_user_claim_merkle_hash = compute_merkle_in_mem_sha256(
            old_claimed_bit_vector,
            &data[offset..(offset + 32 * 64)],
            combined_merkle_index,
            64,
        );
        // make sure the user's start state matches the computed state
        if !known_user_claim_merkle_hash.eq(&old_user_claim_merkle_hash) {
            return Err(ClaimDogeBridgeHelperError::MismatchedUserClaimDeltaMerkleProofOldRoot);
        }

        // mark the transaction wit
        old_claimed_bit_vector[claimed_bit_vector_byte_index] |= claimed_bit_vector_bit_mask;
        let new_user_claim_merkle_hash = compute_merkle_in_mem_sha256(
            old_claimed_bit_vector,
            &data[offset..(offset + 32 * 64)],
            combined_merkle_index,
            64,
        );

        Ok((new_user_claim_merkle_hash, amount))
    }
}

pub struct TransactionInBlockProofV1 {
    pub merkle_proof_siblings: Vec<QHash256>,
    pub transaction: BTCTransaction,
}


impl TransactionInBlockProofV1 {
    pub fn new(
        merkle_proof_siblings: Vec<QHash256>,
        transaction: BTCTransaction,
    ) -> Self {
        TransactionInBlockProofV1 {
            merkle_proof_siblings,
            transaction,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let tx_bytes = self.transaction.to_bytes();
        let size = 1 + self.merkle_proof_siblings.len() * 32 + 4 + tx_bytes.len();
        let mut bytes = Vec::with_capacity(size);
        bytes.push(self.merkle_proof_siblings.len() as u8);
        for sibling in self.merkle_proof_siblings.iter() {
            bytes.extend_from_slice(sibling);
        }
        let tx_len_u32 = tx_bytes.len() as u32;
        bytes.extend_from_slice(&tx_len_u32.to_le_bytes());
        bytes.extend_from_slice(&tx_bytes);

        assert!(bytes.len() == size);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<(Self, usize)> {
        let mut offset = 0;
        let siblings_len = bytes[offset];
        offset += 1;
        let mut merkle_proof_siblings = Vec::with_capacity(siblings_len as usize);
        for _ in 0..siblings_len {
            let mut sibling = [0u8; 32];
            sibling.copy_from_slice(&bytes[offset..offset + 32]);
            offset += 32;
            merkle_proof_siblings.push(sibling);
        }
        let tx_len = u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let transaction = BTCTransaction::from_bytes(&bytes[offset..offset + tx_len])?;
        offset += tx_len;

        Ok((TransactionInBlockProofV1 {
            merkle_proof_siblings,
            transaction,
        }, offset))
    }

    // returns the (tx_hash, tx_bytes, read_length)
    pub fn get_proof_tx_in_block<'a>(
        data: &'a [u8],
        index_in_block: u32,
        known_block_tx_merkle_root: &[u8],
    ) -> QClaimDogeResult<(QHash256, &'a [u8], usize)> {
        if data.len() < MIN_POSSIBLE_TX_SIZE {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }
        if known_block_tx_merkle_root.len() != 32 {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }

        let siblings_len = data[0] as u32;
        if siblings_len >= 30 || index_in_block >= (1u32 << siblings_len) {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }

        let siblings_start = 1;
        let tx_size_start = siblings_start + siblings_len as usize * 32;
        let tx_start = tx_size_start + 4;
        if data.len() <= (tx_start + MIN_POSSIBLE_TX_SIZE) {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }
        let tx_size =
            u32::from_le_bytes(data[tx_size_start..tx_size_start + 4].try_into().unwrap()) as usize;
        if tx_size < MIN_POSSIBLE_TX_SIZE || tx_size > MAX_REASONABLE_TX_SIZE {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }
        if data.len() < tx_start + tx_size {
            return Err(ClaimDogeBridgeHelperError::InvalidTransactionProofV1Blob);
        }

        let tx_hash =
            hash_impl_sha256_bytes(&hash_impl_sha256_bytes(&data[tx_start..tx_start + tx_size]));

        let computed_tx_merkle_root = compute_merkle_in_mem_btc_has256(
            tx_hash,
            &data[siblings_start..tx_size_start],
            index_in_block,
            siblings_len as usize,
        );
        if computed_tx_merkle_root != known_block_tx_merkle_root {
            return Err(ClaimDogeBridgeHelperError::MismatchedTxMerkleRoots);
        }

        Ok((
            tx_hash,
            &data[tx_start..tx_start + tx_size],
            tx_start + tx_size,
        ))
    }

    pub fn check_is_deposit_address_v1_mem(
        tx_data: &[u8],
        output_index: usize,
        solana_public_key: &[u8; 32],
        bridge_public_key_hash: &[u8; 20],
    ) -> QClaimDogeResult<u64> {
        match BTCTransaction::get_output_skip_decode(tx_data, 0, output_index) {
            Ok((version, locktime, output)) => {
                if version != 1 && version != 2 {
                    Err(ClaimDogeBridgeHelperError::InvaildProofTransactionVersion)
                } else if locktime != 0 {
                    Err(ClaimDogeBridgeHelperError::InvaildProofTransactionLocktime)
                } else {
                    if is_bridge_desposit_output_v1_for_user(
                        &output,
                        solana_public_key,
                        bridge_public_key_hash,
                    ) {
                        Ok(output.value)
                    } else {
                        Err(ClaimDogeBridgeHelperError::InvalidProofTransactionOutput)
                    }
                }
            }
            Err(_) => Err(ClaimDogeBridgeHelperError::InvalidProofTransactionData),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn test_v1() -> anyhow::Result<()> {
        Ok(())
    }
}
