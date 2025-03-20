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
use borsh::{BorshSerialize, BorshDeserialize};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

#[cfg(feature = "serde")]
use serde_with::serde_as;

use zerocopy::little_endian::{U16, U32};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, Unaligned};

use crate::{core_data::QHash256, error::{DogeBridgeError, QDogeResult}};


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, IntoBytes, Immutable, Unaligned, Default)]
#[repr(C)]
pub struct BlockDataRecord {
    pub block_hash_tree_root: QHash256,
    pub block_hash: QHash256,
    pub tx_tree_merkle_root: QHash256,
    pub timestamp: U32,
    pub bits: U32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, IntoBytes, Default)]
#[repr(C)]
pub struct PoWBlockContext {
    pub last_height: u32,
    pub last_block_time: u32,
    pub last_bits: u32,
    pub first_block_time: u32,
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, Serialize, Deserialize, IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct BlockDataTracker<const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize, const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize> {
    pub tip_block_number: U32,
    pub tip_internal_index: U16,
    #[serde_as(as = "[_; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE]")]
    pub records: [BlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
}

#[cfg(not(feature = "serde"))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes,  IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct BlockDataTracker<const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize, const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize> {
    pub tip_block_number: U32,
    pub tip_internal_index: U16,
    pub records: [BlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
}


impl<const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize, const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize> BlockDataTracker<QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS> {
    /*
    fn new_empty() -> Self {
        BlockDataTracker {
            tip_block_number: 0,
            tip_internal_index: 0,
            records: [BlockDataRecord {
                block_hash_merkle_tree_root: QHash256::default(),
                block_hash: QHash256::default(),
                tx_tree_merkle_root: QHash256::default(),
                timestamp: 0,
                bits: 0,
            }; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
        }
    }*/
    pub fn new_with_data(
        tip_block_number: u32,
        tip_internal_index: u16,
        records: [BlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE],
    ) -> Self {
        BlockDataTracker {
            tip_block_number: tip_block_number.into(),
            tip_internal_index: tip_internal_index.into(),
            records: records,
        }
    }

    pub fn get_block_hash_if_exists(&self, block_number: u32) -> Option<QHash256> {
        if !self.contains_block(block_number) {
            return None;
        }
        Some(self.get_record_if_exists(block_number)?.block_hash)
    }

    pub fn get_block_hash(&self, block_number: u32) -> QDogeResult<QHash256> {
        if !self.contains_block(block_number) {
            return Err(DogeBridgeError::BlockNotInCache);
        }
        Ok(self.get_record(block_number)?.block_hash)
    }

    pub fn get_pow_context(&self, block_number: u32) -> QDogeResult<PoWBlockContext> {
        if block_number < 2 || !self.contains_block_range(block_number-2, block_number-1) {
            return Err(DogeBridgeError::BlockNotInCache);
        }
        let last_index  = self.get_index_for_block_unchecked(block_number-1);
        let first_index  = self.get_index_for_block_unchecked(block_number-2);
        Ok(PoWBlockContext {
            last_height: block_number-1,
            last_block_time: self.records[last_index].timestamp.into(),
            last_bits: self.records[last_index].bits.into(),
            first_block_time: self.records[first_index].timestamp.into(),
        })
    }
    pub fn get_tip_block_number(&self) -> u32 {
        self.tip_block_number.into()
    }

    pub fn get_finalized_block_number(&self) -> u32 {
        self.get_tip_block_number() - QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS as u32
    }
    pub fn get_tip_internal_index(&self) -> u16 {
        self.tip_internal_index.into()
    }



    pub fn add_record(&mut self, record: BlockDataRecord) {
        let new_tip_index = (self.get_tip_internal_index() as usize +1)%(QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE);
        self.records[new_tip_index] = record;
        self.tip_internal_index = (new_tip_index as u16).into();
        self.tip_block_number += 1;
    }
    pub fn contains_block_range(&self, start_block_number_inclusive: u32, end_block_number_inclusive: u32) -> bool {
        self.tip_block_number <= end_block_number_inclusive && start_block_number_inclusive > self.get_tip_block_number() - QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE as u32
    }
    pub fn contains_block(&self, block_number: u32) -> bool {
        block_number <= self.get_tip_block_number() && block_number > self.get_tip_block_number() - QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE as u32
    }
    fn get_index_for_block_unchecked(&self, block_number: u32) -> usize {
        (QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE + self.get_tip_internal_index() as usize - (self.get_tip_block_number() - block_number) as usize) % QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE
    }

    pub fn get_record_if_exists(&self, block_number: u32) -> Option<BlockDataRecord> {
        if !self.contains_block(block_number) {
            return None;
        }
        Some(self.records[self.get_index_for_block_unchecked(block_number)])
    }

    pub fn get_record(&self, block_number: u32) -> QDogeResult<BlockDataRecord> {
        if !self.contains_block(block_number) {
            return Err(DogeBridgeError::BlockNotInCache);
        }else{
            Ok(self.records[self.get_index_for_block_unchecked(block_number)])
        }
    }
    pub fn rollback_first(&mut self, last_good_block_number: u32, num_blocks_to_insert: usize) -> QDogeResult<()> {
        if last_good_block_number == self.get_tip_block_number() {
            return Ok(());
        }
        if !self.contains_block(last_good_block_number) {
            return Err(DogeBridgeError::BlockNotInCache);
        }
        let offset = (self.get_tip_block_number() - last_good_block_number) as usize;
        if offset >= QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS as usize {
            return Err(DogeBridgeError::AttemptedToModifiyFinalizedBlock);
        }
        if offset < num_blocks_to_insert {
            return Err(DogeBridgeError::InsufficientBlocksProvidedForRollback);
        }
        self.tip_internal_index = (((self.get_tip_internal_index() as usize + QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE - offset as usize) % QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE) as u16).into();
        self.tip_block_number = last_good_block_number.into();
        Ok(())
    }
    pub fn rollback_insert(&mut self, last_good_block_number: u32, blocks: &[BlockDataRecord]) -> QDogeResult<()> {
        self.rollback_first(last_good_block_number, blocks.len())?;
        for block in blocks {
            self.add_record(*block);
        }
        Ok(())
    }

    pub fn get_record_ref_if_exists(&self, block_number: u32) -> Option<&BlockDataRecord> {
        if !self.contains_block(block_number) {
            return None;
        }

        let index = (QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE + self.get_tip_internal_index() as usize - (self.get_tip_block_number() - block_number) as usize) % QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE;
        Some(&self.records[index])
    }


    pub fn get_record_ref(&self, block_number: u32) -> QDogeResult<&BlockDataRecord> {
        if !self.contains_block(block_number) {
            return Err(DogeBridgeError::BlockNotInCache);
        }

        let index = (QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE + self.get_tip_internal_index() as usize - (self.get_tip_block_number() - block_number) as usize) % QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE;
        Ok(&self.records[index])
    }
}