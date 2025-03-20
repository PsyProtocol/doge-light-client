/*
Copyright (C) 2025 Zero Knowledge Labs Limited, QED Protocol

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU Affero General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU Affero General Public License for more details.

You should have received a copy of the GNU Affero General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.

Additional terms under GNU AGPL version 3 section 7:

As permitted by section 7(b) of the GNU Affero General Public License, 
you must retain the following attribution notice in all copies or 
substantial portions of the software:

"This software was created by QED (https://qedprotocol.com)
with contributions from Carter Feldman (https://x.com/cmpeq)."
*/

#[cfg(feature = "borsh")]
use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    block_data_tracker::BlockDataRecord,
    core_data::{QDogeBlock, QDogeBlockHeader, QHash256},
    hash::{
        sha256::QSha256Hasher,
        traits::MerkleHasher,
    },
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InitBlockDataRecord {
    pub block_hash: QHash256,
    pub tx_tree_merkle_root: QHash256,
    pub timestamp: u32,
    pub bits: u32,
}

impl Default for InitBlockDataRecord {
    fn default() -> Self {
        Self {
            block_hash: QHash256::default(),
            tx_tree_merkle_root: QHash256::default(),
            timestamp: 0,
            bits: 0,
        }
    }
}

impl From<InitBlockDataRecord> for BlockDataRecord {
    fn from(x: InitBlockDataRecord) -> BlockDataRecord {
        BlockDataRecord {
            block_hash: x.block_hash,
            tx_tree_merkle_root: x.tx_tree_merkle_root,
            timestamp: x.timestamp.into(),
            bits: x.bits.into(),
            block_hash_tree_root: [0; 32],
        }
    }
}

impl From<&InitBlockDataRecord> for BlockDataRecord {
    fn from(x: &InitBlockDataRecord) -> BlockDataRecord {
        BlockDataRecord {
            block_hash: x.block_hash,
            tx_tree_merkle_root: x.tx_tree_merkle_root,
            timestamp: x.timestamp.into(),
            bits: x.bits.into(),
            block_hash_tree_root: [0; 32],
        }
    }
}

/*
#[cfg(not(feature = "serde"))]
#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct InitBlockDataIBC<const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize, const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize> {
    pub records: [InitBlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
    pub tracker_tree_siblings: [QHash256; QDOGE_BRIDGE_BLOCK_TREE_HEIGHT],
    pub tip_block_number: u32,
}
*/

#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct InitBlockDataIBC<
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
> {
    pub records: [InitBlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
    pub tracker_tree_siblings: [QHash256; QDOGE_BRIDGE_BLOCK_TREE_HEIGHT],
    pub tip_block_number: u32,
}

type QBlockTreeTrackerHasher = QSha256Hasher;

pub fn get_empty_siblings<const N: usize>() -> [QHash256; N] {
    let mut siblings = [QHash256::default(); N];
    let mut current = QHash256::default();
    for i in 1..N {
        current = QBlockTreeTrackerHasher::two_to_one(&current, &current);
        siblings[i] = current;
    }
    siblings
}
impl<
        const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
        const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
    > InitBlockDataIBC<QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>
{
    pub fn new_from_blocks(
        blocks: &[QDogeBlock; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
        tracker_tree_siblings: [QHash256; QDOGE_BRIDGE_BLOCK_TREE_HEIGHT],
        tip_block_number: u32,
    ) -> Self {
        Self {
            records: core::array::from_fn(|i| InitBlockDataRecord {
                block_hash: blocks[i].header.get_hash(),
                tx_tree_merkle_root: blocks[i].header.merkle_root,
                timestamp: blocks[i].header.timestamp,
                bits: blocks[i].header.bits,
            }),
            tracker_tree_siblings,
            tip_block_number,
        }
    }
    pub fn new_from_block_headers(
        headers: &[QDogeBlockHeader; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
        tracker_tree_siblings: [QHash256; QDOGE_BRIDGE_BLOCK_TREE_HEIGHT],
        tip_block_number: u32,
    ) -> Self {
        Self {
            records: core::array::from_fn(|i| InitBlockDataRecord {
                block_hash: headers[i].header.get_hash(),
                tx_tree_merkle_root: headers[i].header.merkle_root,
                timestamp: headers[i].header.timestamp,
                bits: headers[i].header.bits,
            }),
            tracker_tree_siblings,
            tip_block_number,
        }
    }
    pub fn new_from_blocks_empty_tree(
        blocks: &[QDogeBlock; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
        tip_block_number: u32,
    ) -> Self {
        Self::new_from_blocks(
            blocks,
            get_empty_siblings::<QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>(),
            tip_block_number,
        )
    }
    pub fn new_from_block_headers_empty_tree(
        headers: &[QDogeBlockHeader; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
        tip_block_number: u32,
    ) -> Self {
        Self::new_from_block_headers(
            headers,
            get_empty_siblings::<QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>(),
            tip_block_number,
        )
    }
}
