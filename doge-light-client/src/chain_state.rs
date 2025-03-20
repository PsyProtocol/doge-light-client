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


use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::{
    block_data_tracker::{BlockDataRecord, BlockDataTracker}, constants::DogeNetworkConfig, core_data::{QDogeBlockHeader, QHash256}, error::{DogeBridgeError, QDogeResult}, hash::{merkle::fixed_append_tree::FixedMerkleAppendTree, sha256::QSha256Hasher}, init_params::InitBlockDataIBC, logic::check_doge_block::check_block_header_err
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, Immutable, KnownLayout, IntoBytes, Unaligned)]
#[repr(C)]
pub struct QEDDogeChainStateCore<
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize,
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
> {
    pub block_data_tracker:
        BlockDataTracker<QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS>,
    pub block_tree_tracker: FixedMerkleAppendTree<QHash256, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>,
}

type QBlockTreeTrackerHasher = QSha256Hasher;

impl<
        const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
        const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize,
        const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
    >
    QEDDogeChainStateCore<
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >
{
    pub fn new(
        block_data_tracker: BlockDataTracker<
            QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
            QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        >,
        block_tree_tracker: FixedMerkleAppendTree<QHash256, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>,
    ) -> Self {
        Self {
            block_data_tracker,
            block_tree_tracker,
        }
    }

    pub fn contains_block(&self, block_number: u32) -> bool {
        self.block_data_tracker.contains_block(block_number)
    }
    pub fn contains_block_range(
        &self,
        start_block_number_inclusive: u32,
        end_block_number_inclusive: u32,
    ) -> bool {
        self.block_data_tracker
            .contains_block_range(start_block_number_inclusive, end_block_number_inclusive)
    }
    pub fn get_block_hash(&self, block_number: u32) -> QDogeResult<QHash256> {
        self.block_data_tracker.get_block_hash(block_number)
    }
    pub fn get_finalized_block_number(&self) -> u32 {
        self.block_data_tracker.get_finalized_block_number()
    }
    pub fn get_tip_block_number(&self) -> u32 {
        self.block_data_tracker.get_tip_block_number()
    }
    pub fn get_tip_block_hash(&self) -> QHash256 {
        self.block_data_tracker
            .get_block_hash(self.get_tip_block_number())
            .unwrap()
    }
    pub fn get_finalized_block_hash(&self) -> QHash256 {
        self.block_data_tracker
            .get_block_hash(self.get_finalized_block_number())
            .unwrap()
    }
    pub fn ensure_internal_consistency(&self) -> QDogeResult<()> {
        // sanity checks
        if (self.block_data_tracker.get_tip_block_number() + 1)
            != (self.block_tree_tracker.get_next_index()) as u32
        {
            return Err(DogeBridgeError::BlockTipSyncMismatch);
        }

        if self.get_tip_block_hash() != self.block_tree_tracker.get_value() {
            return Err(DogeBridgeError::BlockTipSyncMismatch);
        }
        Ok(())
    }

    pub fn rollback_insert_blocks<NC: DogeNetworkConfig>(
        &mut self,
        last_good_block_number: u32,
        tree_tracker_changed_left_siblings: &[QHash256],
        blocks: &[QDogeBlockHeader],
        known_aux_pow_block_hashes: &[Option<QHash256>],
    ) -> QDogeResult<()> {
        self.ensure_internal_consistency()?;
        if blocks.len() != known_aux_pow_block_hashes.len() {
            return Err(DogeBridgeError::AuxPowMissing);
        }

        let good_record = self.block_data_tracker.get_record(last_good_block_number)?;
        self.block_tree_tracker
            .revert_to_index::<QBlockTreeTrackerHasher>(
                last_good_block_number as u64,
                tree_tracker_changed_left_siblings,
                good_record.block_hash,
            )?;
        if self
            .block_tree_tracker
            .get_root::<QBlockTreeTrackerHasher>()
            != good_record.block_hash_tree_root
        {
            return Err(DogeBridgeError::RollbackBlockTreeRootMismatch);
        }

        if self.block_tree_tracker.next_index != (last_good_block_number + 1) as u64 {
            return Err(DogeBridgeError::RollbackBlockTreeIndexMismatch);
        }

        self.block_data_tracker
            .rollback_first(last_good_block_number, blocks.len())?;
        for (i, (block, optional_aux_pow_hash)) in blocks.iter().zip(known_aux_pow_block_hashes).enumerate() {
            self.append_block::<NC>(last_good_block_number + i as u32 + 1, block, *optional_aux_pow_hash)?;
        }

        self.ensure_internal_consistency()?;
        Ok(())
    }

    pub fn append_block<NC: DogeNetworkConfig>(
        &mut self,
        block_number: u32,
        block_header: &QDogeBlockHeader,
        known_aux_pow_block_hash: Option<QHash256>,
    ) -> QDogeResult<()> {
        if self.contains_block(block_number) {
            return Err(DogeBridgeError::InsertBlockAlreadyInCache);
        } else if self.get_tip_block_number() + 1 != block_number {
            return Err(DogeBridgeError::InsertBlockNotAtTip);
        } else if block_header.header.previous_block_hash != self.get_tip_block_hash() {
            return Err(DogeBridgeError::InvalidParentBlockHash);
        } else if block_header.header.is_aux_pow() && block_header.aux_pow.is_none() {
            return Err(DogeBridgeError::AuxPowMissing);
        } else if !block_header.header.is_aux_pow() && block_header.aux_pow.is_some() {
            return Err(DogeBridgeError::AuxPowNotExpected);
        }

        self.ensure_internal_consistency()?;

        let pow_context = self.block_data_tracker.get_pow_context(block_number)?;
        check_block_header_err::<NC>(
            pow_context.last_height,
            block_header,
            pow_context.last_block_time,
            pow_context.last_bits,
            pow_context.first_block_time,
            known_aux_pow_block_hash,
        )?;

        let new_block_hash = block_header.header.get_hash();

        self.block_tree_tracker
            .append::<QBlockTreeTrackerHasher>(new_block_hash);
        let block_hash_tree_root = self
            .block_tree_tracker
            .get_root::<QBlockTreeTrackerHasher>();

        let block_data_record = BlockDataRecord {
            block_hash_tree_root,
            block_hash: new_block_hash,
            tx_tree_merkle_root: block_header.header.merkle_root,
            timestamp: block_header.header.timestamp.into(),
            bits: block_header.header.bits.into(),
        };

        self.block_data_tracker.add_record(block_data_record);

        self.ensure_internal_consistency()?;

        Ok(())
    }

    
    pub fn from_init_data(
        init_data: &InitBlockDataIBC<
            QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
            QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
        >,
    ) -> Self {
        let tip_block_number = init_data.tip_block_number;

        let start_block = tip_block_number.saturating_sub(QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE as u32 - 1u32);

        let mut append_tree =
            FixedMerkleAppendTree::<QHash256, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>::new_from_hasher::<
                QBlockTreeTrackerHasher,
            >(
                start_block as u64 + 1,
                init_data.tracker_tree_siblings,
                init_data.records[0].block_hash,
            );

        let mut records: [BlockDataRecord; QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE] =
            init_data.records.map(|x| x.into());

        records[0].block_hash_tree_root = append_tree.get_root::<QBlockTreeTrackerHasher>();

        for i in 1..QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE {
            let new_root = append_tree
                .append_delta_merkle_proof::<QBlockTreeTrackerHasher>(records[i].block_hash)
                .new_root;
            records[i].block_hash_tree_root = new_root;
        }
        let block_data_tracker = BlockDataTracker::new_with_data(
            tip_block_number,
            (QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE - 1) as u16,
            records,
        );
    
        Self::new(block_data_tracker, append_tree)
    }


}


#[cfg(test)]
mod tests {
    #[test]
    fn deserialize_state() -> anyhow::Result<()> {

        Ok(())
    }
}