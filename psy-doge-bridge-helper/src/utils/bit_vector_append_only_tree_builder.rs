use doge_light_client::common_types::QHash256;
use crate::utils::append_only_merkle_tree::AppendOnlyMerkleTreeFixed;

pub struct BitVectorAppendOnlyMerkleTreeFixed<const HEIGHT: usize> {
    pub pending_leaf: [u8; 32],
    // pending_byte_bits stores bits for the current byte. Index 0 is LSB, 7 is MSB.
    pub pending_byte_bits: [u8; 8], 
    pub next_bit_index: usize,
    pub next_byte_index_in_pending_leaf: usize,
    // Tracks the absolute position of the next bit to be written
    pub next_global_bit_index: u32,
    pub tree: AppendOnlyMerkleTreeFixed<HEIGHT, 0>,
}

impl<const HEIGHT: usize> BitVectorAppendOnlyMerkleTreeFixed<HEIGHT> {
    pub fn new_from_empty() -> Self {
        Self {
            pending_leaf: [0u8; 32],
            pending_byte_bits: [0u8; 8],
            next_bit_index: 0,
            next_byte_index_in_pending_leaf: 0,
            next_global_bit_index: 0,
            tree: AppendOnlyMerkleTreeFixed::<HEIGHT, 0>::new_from_empty(),
        }
    }

    /// Flushes the current pending leaf to the tree, regardless of how full it is.
    /// Does NOT automatically advance the global index; that is handled by the caller or sequential writes.
    pub fn finalize_leaf(&mut self) {
        // Ensure any pending bits in the current byte are written to pending_leaf
        self.finalize_byte();

        // If we have written any bytes (or the leaf was empty but we are forcing a flush),
        // we append. Note: If completely empty, this appends a zero leaf.
        // We check if we actually have data pending or if this is called explicitly.
        // For the purpose of "skipping", we treat the current leaf as finished.
        
        let leaf_hash: QHash256 = self.pending_leaf.into();
        self.tree.append_leaf(&leaf_hash);

        // Reset state for the next leaf
        self.pending_leaf = [0u8; 32];
        self.pending_byte_bits = [0u8; 8];
        self.next_byte_index_in_pending_leaf = 0;
        self.next_bit_index = 0;
        
        // We align the global index to the start of the next leaf
        // This calculates the start of the next 256-bit block based on the tree's count.
        self.next_global_bit_index = self.tree.next_index * 256;
    }

    /// Packs the 8 bits in `pending_byte_bits` into a byte and writes it to `pending_leaf`.
    pub fn finalize_byte(&mut self) {
        if self.next_bit_index > 0 {
            // Pack bits. Assuming index 0 is LSB (1) and index 7 is MSB (128).
            // This loop constructs the byte from high to low index, shifting appropriately.
            let mut byte = 0u8;
            for i in 0..8 {
                // if pending_byte_bits[i] is 1, we add 2^i
                if self.pending_byte_bits[i] == 1 {
                    byte |= 1 << i;
                }
            }
            
            self.pending_leaf[self.next_byte_index_in_pending_leaf] = byte;
            self.next_byte_index_in_pending_leaf += 1;
            
            // Reset bits
            self.pending_byte_bits = [0u8; 8];
            self.next_bit_index = 0;

            // If we filled the leaf (32 bytes), flush to tree
            if self.next_byte_index_in_pending_leaf == 32 {
                self.finalize_leaf();
            }
        }
    }

    pub fn inc_bit_index(&mut self) {
        self.next_bit_index += 1;
        self.next_global_bit_index += 1;
        
        if self.next_bit_index == 8 {
            self.finalize_byte();
        }
    }

    pub fn set_true_bit_at(&mut self, global_bit_index: u32) {
        assert!(
            global_bit_index >= self.next_global_bit_index, 
            "Cannot set bit at index {} because current index is {}", 
            global_bit_index, self.next_global_bit_index
        );

        // 1. Check for Leaf Transition
        // If the new bit belongs to a future leaf, we must finalize the current one 
        // and skip any empty leaves in between.
        let current_leaf_idx = self.next_global_bit_index / 256;
        let target_leaf_idx = global_bit_index / 256;

        if target_leaf_idx > current_leaf_idx {
            // We must flush the current leaf (even if partial) to the tree
            self.finalize_leaf();
            
            // Skip empty leaves if the gap is large (e.g., jump from leaf 0 to leaf 5)
            // This function handles inserting zero-hashes for the skipped indices.
            self.tree.skip_to_index_efficient(target_leaf_idx);
            
            // Update global index to the start of the new leaf
            self.next_global_bit_index = target_leaf_idx * 256;
        }

        // 2. Check for Byte Transition (within the same leaf)
        // We might be jumping from bit 0 to bit 16 inside the same leaf.
        // We need to flush the current byte if we are moving past it.
        let target_byte_idx = (global_bit_index % 256) / 8;
        
        // If we jumped over bytes, flush the current partial byte
        if (target_byte_idx as usize) > self.next_byte_index_in_pending_leaf {
             self.finalize_byte();
             // Implicitly, the bytes between the old index and new index in `pending_leaf`
             // are already 0 (from initialization), so we just update the pointer.
             self.next_byte_index_in_pending_leaf = target_byte_idx as usize;
        }

        // 3. Set the bit
        // We update the local bit index based on the target
        self.next_bit_index = (global_bit_index % 8) as usize;
        
        // Ensure our global tracker matches exactly where we are writing
        self.next_global_bit_index = global_bit_index;

        self.pending_byte_bits[self.next_bit_index] = 1u8;
        
        // 4. Advance
        self.inc_bit_index();
    }
    pub fn finalize_into_root(mut self) -> QHash256 {
        // Finalize any pending byte and leaf
        self.finalize_leaf();
        self.tree.current_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sequential_bits() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();

        // Set bits 0, 1, 2
        bv.set_true_bit_at(0);
        bv.set_true_bit_at(1);
        bv.set_true_bit_at(2);

        // Expected: Byte 0 = 00000111 = 0x07
        // State check before finalize
        assert_eq!(bv.pending_byte_bits[0], 1);
        assert_eq!(bv.pending_byte_bits[1], 1);
        assert_eq!(bv.pending_byte_bits[2], 1);
        assert_eq!(bv.pending_byte_bits[3], 0);

        bv.finalize_leaf();

        // Tree should have 1 leaf
        assert_eq!(bv.tree.next_index, 1);
    }

    #[test]
    fn test_skip_bits_within_byte() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();

        // Set bit 0 and bit 7
        bv.set_true_bit_at(0);
        bv.set_true_bit_at(7);

        // This triggers finalize_byte inside set_true_bit_at (via inc_bit_index)
        // 0x01 | 0x80 = 0x81 = 129
        
        // Manually flush to check buffer
        bv.finalize_leaf(); 
        
        // We can't easily peek into the tree's hash history without a getter, 
        // but we can verify the indices progressed.
        assert_eq!(bv.tree.next_index, 1);
        assert_eq!(bv.next_global_bit_index, 256);
    }

    #[test]
    fn test_skip_bytes_within_leaf() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();

        // Set bit 0 (Byte 0)
        bv.set_true_bit_at(0);
        
        // Set bit 16 (Byte 2, bit 0)
        // Byte 1 should be skipped (0x00)
        bv.set_true_bit_at(16);

        // Check internal state
        assert_eq!(bv.next_byte_index_in_pending_leaf, 2);
        assert_eq!(bv.pending_byte_bits[0], 1); // Bit 0 of Byte 2 is set

        bv.finalize_leaf();
        
        // Tree index should increment
        assert_eq!(bv.tree.next_index, 1);
    }

    #[test]
    fn test_leaf_boundary_transition() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();

        // Set last bit of first leaf
        bv.set_true_bit_at(255);
        
        // At this point, set_true_bit calls inc_bit_index -> finalize_byte -> finalize_leaf
        // because we hit the end of the 32nd byte.
        assert_eq!(bv.tree.next_index, 1);
        assert_eq!(bv.next_global_bit_index, 256);

        // Set first bit of next leaf
        bv.set_true_bit_at(256);
        
        // Should be working on second leaf
        assert_eq!(bv.tree.next_index, 1); // Not committed yet
        assert_eq!(bv.next_global_bit_index, 257);
    }

    #[test]
    fn test_skip_entire_leaf() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();

        bv.set_true_bit_at(0);
        
        // Skip Leaf 0 (rest of it) and Leaf 1 entirely. Start writing in Leaf 2.
        // Index 512 is the start of Leaf 2 (256 * 2).
        bv.set_true_bit_at(512);

        // Leaf 0 should be appended (index 1)
        // Leaf 1 should be appended as zero (index 2)
        // Currently building Leaf 2 (next_index still 2 until flushed)
        assert_eq!(bv.tree.next_index, 2);
        assert_eq!(bv.next_global_bit_index, 513);
        
        bv.finalize_leaf();
        assert_eq!(bv.tree.next_index, 3);
    }

    #[test]
    #[should_panic]
    fn test_backwards_write() {
        const HEIGHT: usize = 5;
        let mut bv = BitVectorAppendOnlyMerkleTreeFixed::<HEIGHT>::new_from_empty();
        bv.set_true_bit_at(100);
        bv.set_true_bit_at(99);
    }
}