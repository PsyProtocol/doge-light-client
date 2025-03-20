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

use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::hash::traits::{get_zero_hashes, MerkleHasher, MerkleZeroHasher, ZeroableHash};

use super::{delta_merkle_proof::DeltaMerkleProofCore, merkle_proof::{MerkleProofCore, MerkleProofCorePartial}};


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned)]
#[repr(C)]
pub struct MerkleAppendTreeLevel<Hash: PartialEq + Copy> {
    pub left: Hash,
    pub right: Hash,
    pub zero_hash: Hash,
}

impl<Hash: PartialEq + Copy> MerkleAppendTreeLevel<Hash> {
    pub fn get_left(&self) -> Hash {
        self.left
    }
    pub fn get_right(&self) -> Hash {
        self.right
    }
    pub fn get_zero_hash(&self) -> Hash {
        self.zero_hash
    }
    pub fn get_hash<H: MerkleHasher<Hash>>(&self) -> Hash {
        H::two_to_one(&self.left, &self.right)
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MerkleAppendTree<Hash: PartialEq + Copy> {
    pub height: u8,
    pub next_index: u64,
    pub levels: Vec<MerkleAppendTreeLevel<Hash>>,
}


impl<Hash: PartialEq + Copy + ZeroableHash> MerkleAppendTree<Hash> {

    pub fn new_empty<Hasher: MerkleZeroHasher<Hash>>(
        height: u8,
    ) -> Self {
        let zero_hashes = get_zero_hashes::<Hash, Hasher>(height as usize);
        Self {
            height,
            next_index: 0,
            levels: zero_hashes.into_iter().map(|zh| MerkleAppendTreeLevel {
                left: zh,
                right: zh,
                zero_hash: zh,
            }).collect(),
        }
    }
    pub fn new_from_hasher<Hasher: MerkleZeroHasher<Hash>>(
        next_index: u64,
        siblings: Vec<Hash>,
        value: Hash,
    ) -> Self {
        let zero_hashes = get_zero_hashes::<Hash, Hasher>(siblings.len());
        Self::new::<Hasher>(next_index, zero_hashes, siblings, value)
    }
}
impl<Hash: PartialEq + Copy> MerkleAppendTree<Hash> {
    pub fn new<Hasher: MerkleHasher<Hash>>(
        next_index: u64,
        zero_hashes: Vec<Hash>,
        siblings: Vec<Hash>,
        value: Hash,
    ) -> Self {
        assert_eq!(zero_hashes.len(), siblings.len());

        if next_index == 0 {
            Self {
                height: siblings.len() as u8,
                next_index: 0,
                levels: zero_hashes.into_iter().map(|zh| MerkleAppendTreeLevel {
                    left: zh,
                    right: zh,
                    zero_hash: zh,
                }).collect(),
            }
        }else{
            let mut levels = Vec::with_capacity(siblings.len());

            let mut current = value;
            let mut current_index = next_index-1;
            for (sibling, zero_hash) in siblings.into_iter().zip(zero_hashes.into_iter()) {
                let swap = (current_index & 1) == 1;
                let new_v = Hasher::two_to_one_swap(swap, &current, &sibling);

                if swap {
                    levels.push(MerkleAppendTreeLevel {
                        left: sibling,
                        right: current,
                        zero_hash: zero_hash,
                    });
                }else{
                    levels.push(MerkleAppendTreeLevel {
                        left: current,
                        right: sibling,
                        zero_hash: zero_hash,
                    });
                }

                current = new_v;
                
                current_index >>= 1;
            }
            Self {
                height: levels.len() as u8,
                next_index,
                levels,
            }

        }
    }
    pub fn get_next_index(&self) -> u64 {
        self.next_index
    }
    pub fn get_height(&self) -> u8 {
        self.height
    }
    pub fn get_value(&self) -> Hash {
        let is_next_right = (self.next_index&1) == 1;
        if is_next_right{
            self.levels[0].left
        }else{
            self.levels[0].right
        }
    }
    pub fn get_root<Hasher: MerkleHasher<Hash>>(&self) -> Hash {
        self.levels.last().unwrap().get_hash::<Hasher>()
    }

    pub fn append<H: MerkleHasher<Hash>>(&mut self, new_value: Hash) {
        let mut current = new_value;
        let mut current_index = self.next_index;

        for level in self.levels.iter_mut() {
            let is_right_child = (current_index & 1) == 1;
            if is_right_child {
                level.right = current;
            }else{
                level.left = current;
                level.right = level.zero_hash;
            }
            current = level.get_hash::<H>();
            current_index >>= 1;
        }
        self.next_index += 1;
    }

    pub fn append_delta_merkle_proof<H: MerkleHasher<Hash>>(&mut self, new_value: Hash) -> DeltaMerkleProofCore<Hash> {
        let mut current = new_value;
        let mut current_index = self.next_index;

        for level in self.levels.iter_mut() {
            let is_right_child = (current_index & 1) == 1;
            if is_right_child {
                level.right = current;
            }else{
                level.left = current;
                level.right = level.zero_hash;
            }
            current = level.get_hash::<H>();
            current_index >>= 1;
        }
        self.next_index += 1;

        let zero_leaf = self.levels[0].zero_hash;

        let mpp = self.get_partial_merkle_proof_for_current_index();

        DeltaMerkleProofCore::from_params::<H>(
            mpp.index,
            zero_leaf,
            new_value,
            mpp.siblings
        )
    }

    pub fn get_partial_merkle_proof_for_current_index(&self) -> MerkleProofCorePartial<Hash> {
        if self.next_index == 0 {
            MerkleProofCorePartial::new_from_params(0, self.get_value(), self.levels.iter().map(|x|x.zero_hash).collect())
        }else{
            let mut siblings = Vec::with_capacity(self.height as usize);
            let value = self.get_value();
            let index = self.next_index-1;
            let mut current_index = index;

            for level in self.levels.iter() {
                let is_sibling_left_child = (current_index & 1) == 1;
                if is_sibling_left_child {
                    siblings.push(level.left);
                }else{
                    siblings.push(level.right);
                }
                current_index >>= 1;
            }
            MerkleProofCorePartial::new_from_params(index, value, siblings)
        }
    }

    pub fn get_merkle_proof_for_current_index<H: MerkleHasher<Hash>>(&self) -> MerkleProofCore<Hash> {
        if self.next_index == 0 {
            MerkleProofCore::new_from_params::<H>(0, self.get_value(), self.levels.iter().map(|x|x.zero_hash).collect())
        }else{
            let mut siblings = Vec::with_capacity(self.height as usize);
            let value = self.get_value();
            let index = self.next_index-1;
            let mut current_index = index;

            for level in self.levels.iter() {
                let is_sibling_left_child = (current_index & 1) == 1;
                if is_sibling_left_child {
                    siblings.push(level.left);
                }else{
                    siblings.push(level.right);
                }
                current_index >>= 1;
            }
            MerkleProofCore::new_from_params::<H>(index, value, siblings)
        }
    }

}


