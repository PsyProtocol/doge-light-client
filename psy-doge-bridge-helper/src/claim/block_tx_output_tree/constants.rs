pub const TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH: usize = 28;
pub const TXO_TREE_INDEX_BITS_TX_NUM_LENGTH: usize = 13;
pub const TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH: usize = 4;
pub const TXO_TREE_LEAF_BIT_INDEX_LENGTH: usize = 8;
pub const TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH: usize = TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH + TXO_TREE_LEAF_BIT_INDEX_LENGTH;

pub const TXO_MERKLE_INDEX_TOTAL_BITS: usize =
    TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH +
    TXO_TREE_INDEX_BITS_TX_NUM_LENGTH +
    TXO_TREE_INDEX_BITS_TOP_OUTPUT_NUM_LENGTH;
pub const TXO_MERKLE_TREE_HEIGHT: usize = TXO_MERKLE_INDEX_TOTAL_BITS;

pub const TXO_MERKLE_INDEX_MASK: u64 = (1u64 << TXO_MERKLE_INDEX_TOTAL_BITS) - 1;
pub const TXO_MERKLE_TREE_MAX_INDEX: u64 = (1u64 << TXO_MERKLE_INDEX_TOTAL_BITS) - 1;

pub const TXO_COMBINED_INDEX_TOTAL_BITS: usize =
    TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH +
    TXO_TREE_INDEX_BITS_TX_NUM_LENGTH +
    TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH;

pub const TXO_COMBINED_INDEX_MASK: u64 = (1u64 << TXO_COMBINED_INDEX_TOTAL_BITS) - 1;
pub const TXO_COMBINED_INDEX_MAX_VALUE: u64 = (1u64 << TXO_COMBINED_INDEX_TOTAL_BITS) - 1;



pub const TXO_TREE_MAX_BLOCK_NUMBER: u32 = (1u32 << TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH) - 1;
pub const TXO_TREE_MAX_TX_PER_BLOCK: u16 = (1u16 << TXO_TREE_INDEX_BITS_TX_NUM_LENGTH) - 1;
pub const TXO_TREE_MAX_OUTPUTS_PER_TX: u16 = (1u16 << (TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) - 1;

pub const TXO_TREE_INDEX_BLOCK_NUM_LOWERED_MASK: u64 = (1u64 << TXO_TREE_INDEX_BITS_BLOCK_NUM_LENGTH) - 1;
pub const TXO_TREE_INDEX_TX_NUM_LOWERED_MASK: u64 = (1u64 << TXO_TREE_INDEX_BITS_TX_NUM_LENGTH) - 1;
pub const TXO_TREE_INDEX_OUTPUT_NUM_LOWERED_MASK: u64 = (1u64 << TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) - 1;



#[inline(always)]
pub const fn is_valid_txo_combined_index(index: u64) -> bool {
    index & TXO_COMBINED_INDEX_MASK == index
}

#[inline(always)]
pub const fn is_valid_txo_merkle_index(index: u64) -> bool {
    index & TXO_MERKLE_INDEX_MASK == index
}


#[inline(always)]
pub const fn get_txo_block_number_tx_number_output_leaf_merkle_index(index: u64) -> (u32, u16, u8) {
    let block_num = ((index >> (TXO_TREE_INDEX_BITS_TX_NUM_LENGTH + TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) & TXO_TREE_INDEX_BLOCK_NUM_LOWERED_MASK) as u32;
    let tx_num = ((index >> TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) & TXO_TREE_INDEX_TX_NUM_LOWERED_MASK) as u16;
    let output_num = (index & TXO_TREE_INDEX_OUTPUT_NUM_LOWERED_MASK) as u8;
    (block_num, tx_num, output_num)
}



#[inline(always)]
pub const fn get_txo_merkle_index(block_num: u32, tx_num: u16, output_leaf_index: u8) -> u64 {
    ((block_num as u64) << (TXO_TREE_INDEX_BITS_TX_NUM_LENGTH + TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) |
    ((tx_num as u64) << TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) |
    (output_leaf_index as u64)
}

#[inline(always)]
pub const fn get_txo_merkle_index_with_output_index(block_num: u32, tx_num: u16, output_index_in_tx: u16) -> u64 {
    get_txo_merkle_index(block_num, tx_num, (output_index_in_tx >> 8) as u8)
}
#[inline(always)]
pub const fn get_txo_merkle_index_and_bit_index(block_num: u32, tx_num: u16, output_index_in_tx: u16) -> (u64, u8) {
    (
        get_txo_merkle_index(block_num, tx_num, (output_index_in_tx >> 8) as u8),
        (output_index_in_tx & 0xFF) as u8,
    )
}
#[inline(always)]
pub const fn get_txo_combined_index(block_num: u32, tx_num: u16, output_index_in_tx: u16) -> u64 {
    ((block_num as u64) << (TXO_TREE_INDEX_BITS_TX_NUM_LENGTH + TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) |
    ((tx_num as u64) << TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) |
    (output_index_in_tx as u64)
}
#[inline(always)]
pub const fn get_txo_combined_index_for_merkle_index_bit_index(block_num: u32, tx_num: u16, output_merkle_index: u8, output_bit_index: u8) -> u64 {
    ((block_num as u64) << (TXO_TREE_INDEX_BITS_TX_NUM_LENGTH + TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) |
    ((tx_num as u64) << TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) |
    (((output_merkle_index as u64) << 8) | (output_bit_index as u64))
}

#[inline(always)]
pub const fn get_txo_block_number_tx_number_output_index_from_combined_index(index: u64) -> (u32, u16, u16) {
    let block_num = ((index >> (TXO_TREE_INDEX_BITS_TX_NUM_LENGTH + TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH)) & TXO_TREE_INDEX_BLOCK_NUM_LOWERED_MASK) as u32;
    let tx_num = ((index >> TXO_TREE_COMBINED_INDEX_BITS_OUTPUT_NUM_LENGTH) & TXO_TREE_INDEX_TX_NUM_LOWERED_MASK) as u16;
    let output_index = (index & TXO_TREE_INDEX_OUTPUT_NUM_LOWERED_MASK) as u16;
    (block_num, tx_num, output_index)
}
#[inline(always)]
pub const fn is_valid_block_num_tx_num_output_index(block_num: u32, tx_num: u16, output_index_in_tx: u16) -> bool {
    (block_num & (TXO_TREE_INDEX_BLOCK_NUM_LOWERED_MASK as u32)) == block_num &&
    (tx_num & (TXO_TREE_INDEX_TX_NUM_LOWERED_MASK as u16)) == tx_num &&
    (output_index_in_tx & (TXO_TREE_INDEX_OUTPUT_NUM_LOWERED_MASK as u16)) == output_index_in_tx
}


#[inline(always)]
pub const fn is_valid_siblings_length_for_txo_merkle_proof(siblings_length: usize) -> bool {
    siblings_length == TXO_MERKLE_TREE_HEIGHT
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_txo_constants() {
        assert_eq!(TXO_MERKLE_INDEX_TOTAL_BITS, 45);
        assert_eq!(TXO_MERKLE_TREE_HEIGHT, 45);
        assert_eq!(TXO_COMBINED_INDEX_TOTAL_BITS, 53);
    }
    #[test]
    fn test_txo_index_functions() {
        // ~450 years of dogecoin blocks at 1 block per minute
        let block_num = 236_520_000;

        // max tx per block is around ~5000 based on 1mb blocks
        let tx_num = 6000u16;
        // max outputs per tx is around ~2777 based on 4,000,000 weight limit
        let output_index_in_tx = 2800u16;
        assert!(is_valid_block_num_tx_num_output_index(block_num, tx_num, output_index_in_tx));

        let merkle_index = get_txo_merkle_index_with_output_index(block_num, tx_num, output_index_in_tx);
        let (bn, tn, on) = get_txo_block_number_tx_number_output_leaf_merkle_index(merkle_index);
        assert_eq!(bn, block_num);
        assert_eq!(tn, tx_num);
        assert_eq!(on, (output_index_in_tx >> 8) as u8);

        let combined_index = get_txo_combined_index(block_num, tx_num, output_index_in_tx);
        let (bn2, tn2, oi2) = get_txo_block_number_tx_number_output_index_from_combined_index(combined_index);
        assert_eq!(bn2, block_num);
        assert_eq!(tn2, tx_num);
        assert_eq!(oi2, output_index_in_tx);

        let (mi, bi) = get_txo_merkle_index_and_bit_index(block_num, tx_num, output_index_in_tx);
        assert_eq!(mi, merkle_index);
        assert_eq!(bi, (output_index_in_tx & 0xFF) as u8);

        // round trip
        let combined_index_2 = get_txo_combined_index_for_merkle_index_bit_index(block_num, tx_num, (output_index_in_tx >> 8) as u8, (output_index_in_tx & 0xFF) as u8);
        assert_eq!(combined_index, combined_index_2);

        let invalid_block_num = TXO_TREE_MAX_BLOCK_NUMBER + 1;
        assert!(!is_valid_block_num_tx_num_output_index(invalid_block_num, tx_num, output_index_in_tx));

        let invalid_tx_num = TXO_TREE_MAX_TX_PER_BLOCK + 1;
        assert!(!is_valid_block_num_tx_num_output_index(block_num, invalid_tx_num, output_index_in_tx));

        let invalid_output_index = TXO_TREE_MAX_OUTPUTS_PER_TX + 1;
        assert!(!is_valid_block_num_tx_num_output_index(block_num, tx_num, invalid_output_index));
    }

}