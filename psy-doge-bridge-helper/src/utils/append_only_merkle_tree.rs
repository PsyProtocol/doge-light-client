use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_two_to_one_bytes};

use crate::utils::sha256_zero_hashes::SHA256_ZERO_HASHES;

pub struct AppendOnlyMerkleTreeFixed<const HEIGHT: usize, const LEAF_HEIGHT: usize> {
    pub start_root: QHash256,
    pub current_root: QHash256,
    pub next_siblings: [QHash256; HEIGHT],
    pub next_index: u32,
}

// Append Only Merkle Tree Builder for Auto Claim Deposits Tree
impl<const HEIGHT: usize, const LEAF_HEIGHT: usize> AppendOnlyMerkleTreeFixed<HEIGHT, LEAF_HEIGHT> {
    pub fn new_from_empty() -> Self {
        let siblings = core::array::from_fn(|i| SHA256_ZERO_HASHES[LEAF_HEIGHT+ i]);
        Self {
            start_root: SHA256_ZERO_HASHES[LEAF_HEIGHT + HEIGHT],
            current_root: SHA256_ZERO_HASHES[LEAF_HEIGHT + HEIGHT],
            next_siblings: siblings,
            next_index: 0,
        }
    }

    pub fn new_from_siblings(
        last_siblings: &[QHash256],
        last_index: u32,
        last_value: &QHash256,
    ) -> anyhow::Result<Self> {
        let is_last_value_empty = last_value == &SHA256_ZERO_HASHES[LEAF_HEIGHT];
        if last_index == 0 && is_last_value_empty {
            return Ok(Self::new_from_empty());
        } else if is_last_value_empty {
            return Err(anyhow::anyhow!("Last value cannot be empty if last index is not zero"));
        }
        if last_siblings.len() != HEIGHT {
            return Err(anyhow::anyhow!("Invalid siblings length"));
        }

        let mut current = *last_value;
        let mut next_siblings = core::array::from_fn(|i| SHA256_ZERO_HASHES[i+LEAF_HEIGHT]);
        let mut index = last_index;

        for (i, sibling) in last_siblings.iter().enumerate() {
            if (index & 1) == 0 {
                // Case: Left Child
                // We are on the left. We are the "frontier" waiting for a future right child.
                // Store `current` so the next node (which might be our right sibling) can find us.
                next_siblings[i] = current;
                current = hash_impl_sha256_two_to_one_bytes(&current, sibling);
            } else {
                // Case: Right Child
                // We are on the right. The provided `sibling` is the Left Child.
                // That Left Child is the one that was waiting on the frontier. 
                // We must restore it to `next_siblings` because the next index to be added 
                // might still be within this subtree (e.g., 14->15 sharing same parents at L1, L2).
                next_siblings[i] = *sibling;
                current = hash_impl_sha256_two_to_one_bytes(sibling, &current);
            }
            index /= 2;
        }

        let start_root = current;

        Ok(Self {
            start_root,
            current_root: current,
            next_siblings,
            next_index: last_index + 1,
        })
    }

    pub fn append_leaf(&mut self, leaf: &QHash256) {
        let mut current = *leaf;
        let mut index = self.next_index;

        for i in 0..HEIGHT {
            if (index & 1) == 0 {
                // Left Child:
                // Store self as the waiting sibling for the future Right Child.
                self.next_siblings[i] = current;
                
                // Hash with Zero to compute the temporary root for this state
                current = hash_impl_sha256_two_to_one_bytes(&current, &SHA256_ZERO_HASHES[i + LEAF_HEIGHT]);
            } else {
                // Right Child:
                // Retrieve the waiting Left Child
                let left = self.next_siblings[i];
                
                // Merge
                current = hash_impl_sha256_two_to_one_bytes(&left, &current);
                
                // (Optional) We could clear self.next_siblings[i] here, but it will 
                // just be overwritten when the next Left Child at this level appears.
            }
            index /= 2;
        }

        self.current_root = current;
        self.next_index += 1;
    }
    pub fn skip_to_index_efficient(&mut self, new_next_index: u32) {
        assert!(new_next_index >= self.next_index, "Cannot skip backwards");

        if new_next_index == self.next_index {
            return;
        }

        let old_index = self.next_index;
        let mut current_hash = SHA256_ZERO_HASHES[LEAF_HEIGHT];

        for i in 0..HEIGHT {
            let old_bit = (old_index >> i) & 1;
            let new_bit = (new_next_index >> i) & 1;

            // If we were on the right in the old path, we need the sibling to compute 
            // the root path for the old index (bubbling up `current_hash`).
            // We must read this before potentially overwriting it below.
            let old_sibling_if_needed = if old_bit == 1 {
                self.next_siblings[i]
            } else {
                SHA256_ZERO_HASHES[i + LEAF_HEIGHT] // Dummy value, won't be used
            };

            // Update next_siblings for the new index
            if new_bit == 1 {
                // We are on the Right child for the new index.
                // We need to ensure next_siblings[i] holds the correct Left Child.

                // Check if we are still within the same parent context at the next level up.
                // If the bits above this level are the same, we share the same ancestor.
                let intersection = ((old_index ^ new_next_index) >> (i + 1)) == 0;

                if intersection {
                    if old_bit == 0 {
                        // Case: Transition from Left to Right within the same parent.
                        // The `current_hash` (which represents the completed subtree at `old_index` filled with zeros)
                        // becomes the Left Sibling.
                        self.next_siblings[i] = current_hash;
                    } else {
                        // Case: Transition from Right to Right within the same parent.
                        // The Left Sibling was already set correctly. No change needed.
                    }
                } else {
                    // Case: We jumped to a completely different subtree structure.
                    // The Left Sibling for this new position corresponds to a subtree that was entirely skipped.
                    // Therefore, it is a Zero Hash.
                    self.next_siblings[i] = SHA256_ZERO_HASHES[i + LEAF_HEIGHT];
                }
            }
            // If new_bit == 0, we are on a Left child. `next_siblings[i]` is strictly for 
            // storage when waiting for a Right child. We can leave it as garbage or old data 
            // because `append_leaf` will overwrite it before reading.

            // Bubble up `current_hash` to the next level (simulating the path of `old_index`)
            if old_bit == 0 {
                // Old path was Left: Merge with Zero (since we are skipping/padding)
                current_hash = hash_impl_sha256_two_to_one_bytes(&current_hash, &SHA256_ZERO_HASHES[i + LEAF_HEIGHT]);
            } else {
                // Old path was Right: Merge with the saved sibling
                current_hash = hash_impl_sha256_two_to_one_bytes(&old_sibling_if_needed, &current_hash);
            }
        }

        self.current_root = current_hash;
        self.next_index = new_next_index;
    }
    pub fn skip_to_index(&mut self, new_next_index: u32) {
        // make this more efficient
        assert!(new_next_index >= self.next_index);
        while self.next_index < new_next_index {
            self.append_leaf(&SHA256_ZERO_HASHES[0]);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn sample_leaf(i: u64) -> QHash256 {
        let mut bytes = [0u8; 32];
        bytes[..8].copy_from_slice(&i.to_le_bytes());
        bytes.into()
    }

    fn naive_root(leaves: &[QHash256], height: usize) -> QHash256 {
        if leaves.is_empty() {
            return SHA256_ZERO_HASHES[height];
        }

        let mut level = leaves.to_vec();
        let mut depth = 0usize;

        while level.len() > 1 {
            if level.len() % 2 == 1 {
                level.push(SHA256_ZERO_HASHES[depth]);
            }

            let mut next = Vec::with_capacity(level.len() / 2);
            for pair in level.chunks_exact(2) {
                next.push(hash_impl_sha256_two_to_one_bytes(&pair[0], &pair[1]));
            }

            level = next;
            depth += 1;
        }

        // We may not have reached the full tree height yet; keep hashing with zero levels.
        let mut root = level[0];
        while depth < height {
            root = hash_impl_sha256_two_to_one_bytes(&root, &SHA256_ZERO_HASHES[depth]);
            depth += 1;
        }
        root
    }

    fn naive_path(leaves: &[QHash256], index: usize, height: usize) -> Vec<QHash256> {
        let mut level = leaves.to_vec();
        let mut idx = index;
        let mut depth = 0usize;
        let mut path = Vec::with_capacity(height);

        while depth < height {
            if level.is_empty() {
                path.push(SHA256_ZERO_HASHES[depth]);
            } else {
                if level.len() % 2 == 1 {
                    level.push(SHA256_ZERO_HASHES[depth]);
                }
                let sibling_idx = idx ^ 1;
                path.push(level[sibling_idx]);

                let mut next = Vec::with_capacity(level.len() / 2);
                for pair in level.chunks_exact(2) {
                    next.push(hash_impl_sha256_two_to_one_bytes(&pair[0], &pair[1]));
                }
                level = next;
                idx >>= 1;
            }
            depth += 1;
        }

        path
    }

    #[test]
    fn append_matches_naive_for_first_64_leaves() {
        const HEIGHT: usize = 12;
        let mut builder = AppendOnlyMerkleTreeFixed::<HEIGHT, 0>::new_from_empty();
        let mut leaves = Vec::new();

        for i in 0..64 {
            let leaf = sample_leaf(i);
            leaves.push(leaf);
            builder.append_leaf(&leaf);

            assert_eq!(
                builder.current_root,
                naive_root(&leaves, HEIGHT),
                "root mismatch after inserting leaf {}",
                i
            );
        }
    }

    #[test]
    fn new_from_siblings_recovers_state_and_allows_more_appends() {
        const HEIGHT: usize = 32;
        let mut leaves = Vec::new();
        for i in 0..25 {
            leaves.push(sample_leaf(i));
        }

        let last_index = (leaves.len() - 1) as u32;
        let last_leaf = leaves[last_index as usize];
        let siblings = naive_path(&leaves, last_index as usize, HEIGHT);

        let mut rebuilt = AppendOnlyMerkleTreeFixed::<HEIGHT, 0>::new_from_siblings(
            &siblings,
            last_index,
            &last_leaf,
        )
        .expect("builder reconstruction");

        assert_eq!(rebuilt.current_root, naive_root(&leaves, HEIGHT));
        assert_eq!(rebuilt.next_index, last_index + 1);

        for i in 25..40 {
            let leaf = sample_leaf(i);
            leaves.push(leaf);
            rebuilt.append_leaf(&leaf);

            assert_eq!(
                rebuilt.current_root,
                naive_root(&leaves, HEIGHT),
                "root mismatch after rebuilding + appending leaf {}",
                i
            );
        }
    }

    #[test]
    fn empty_builder_matches_zero_root() {
        const HEIGHT: usize = 12;
        let builder = AppendOnlyMerkleTreeFixed::<HEIGHT, 0>::new_from_empty();
        assert_eq!(
            builder.current_root,
            SHA256_ZERO_HASHES[HEIGHT]
        );
        assert_eq!(builder.next_index, 0);
    }
}