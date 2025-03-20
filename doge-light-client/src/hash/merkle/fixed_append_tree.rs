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



use zerocopy::little_endian::U64;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned};

use crate::{error::{DogeBridgeError, QDogeResult}, hash::traits::{get_zero_hashes_sized, MerkleHasher, MerkleZeroHasher, ZeroableHash}};

use super::{append_tree::MerkleAppendTreeLevel, delta_merkle_proof::DeltaMerkleProofCore, merkle_proof::{MerkleProofCore, MerkleProofCorePartial}};


/* 
#[cfg(feature = "serde")]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, Immutable, IntoBytes, KnownLayout, Unaligned, Serialize, Deserialize)]
#[repr(C)]
#[serde_as]
pub struct FixedMerkleAppendTree<Hash: PartialEq + Copy, const HEIGHT: usize> where Hash: serde::Serialize, Hash: serde::de::DeserializeOwned {
    pub next_index: U64,
    #[serde_as(as = "[_; HEIGHT]")]
    pub levels: [MerkleAppendTreeLevel<Hash>; HEIGHT],
}*/

#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, KnownLayout, IntoBytes, Unaligned, Immutable)]
#[repr(C)]
pub struct FixedMerkleAppendTree<Hash: PartialEq + Copy, const HEIGHT: usize> {
    pub next_index: U64,
    pub levels: [MerkleAppendTreeLevel<Hash>; HEIGHT],
}

#[cfg(feature = "serde")]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromBytes, Serialize, Deserialize)]
#[serde(bound = "for<'de2> Hash: Deserialize<'de2>")]
struct SerFixedMerkleAppendTree<Hash: PartialEq + Copy + Serialize, const HEIGHT: usize> {
    pub next_index: U64,
    #[serde_as(as = "[_; HEIGHT]")]
    pub levels: [MerkleAppendTreeLevel<Hash>; HEIGHT],
}


#[cfg(feature = "serde")]
impl<Hash: PartialEq + Copy, const HEIGHT: usize> serde::Serialize for FixedMerkleAppendTree<Hash, HEIGHT> where Hash: serde::Serialize + serde::de::DeserializeOwned {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        serde::Serialize::serialize(&SerFixedMerkleAppendTree {
            next_index: self.next_index,
            levels: self.levels,
        }, serializer)
    }
}



#[cfg(feature = "serde")]
impl<'de, Hash: PartialEq + Copy, const HEIGHT: usize> serde::Deserialize<'de> for FixedMerkleAppendTree<Hash, HEIGHT> where Hash: serde::Serialize + serde::de::DeserializeOwned {
    
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let SerFixedMerkleAppendTree { next_index, levels } = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self {
            next_index,
            levels,
        })
    }
}



impl<Hash: PartialEq + Copy + ZeroableHash, const HEIGHT: usize> FixedMerkleAppendTree<Hash, HEIGHT> {

    pub fn new_empty<Hasher: MerkleHasher<Hash>>() -> Self {
        let zero_hashes = get_zero_hashes_sized::<Hash, Hasher, HEIGHT>();
        Self {
            next_index: 0.into(),
            levels: core::array::from_fn(|i| MerkleAppendTreeLevel {
                left: zero_hashes[i],
                right: zero_hashes[i],
                zero_hash: zero_hashes[i],
            }),
        }
    }
    pub fn new_from_hasher<Hasher: MerkleZeroHasher<Hash>>(
        next_index: u64,
        siblings: [Hash; HEIGHT],
        value: Hash,
    ) -> Self {
        let zero_hashes = get_zero_hashes_sized::<Hash, Hasher, HEIGHT>();
        Self::new::<Hasher>(next_index, zero_hashes, siblings, value)
    }
}
impl<Hash: PartialEq + Copy, const HEIGHT: usize> FixedMerkleAppendTree<Hash, HEIGHT> {
    pub fn new_vec<Hasher: MerkleHasher<Hash>>(
        next_index: u64,
        zero_hashes: Vec<Hash>,
        siblings: Vec<Hash>,
        value: Hash,
    ) -> Self {
        assert_eq!(zero_hashes.len(), siblings.len());
        assert_eq!(siblings.len(), HEIGHT);
        

        Self::new::<Hasher>(next_index, match zero_hashes.try_into() {
            Ok(x) => x,
            Err(_) => panic!("Invalid zero_hashes length"),
        }, match siblings.try_into(){
            Ok(x) => x,
            Err(_) => panic!("invalid siblings length"),
        }, value)


    }
    pub fn new<Hasher: MerkleHasher<Hash>>(
        next_index: u64,
        zero_hashes: [Hash; HEIGHT],
        siblings: [Hash; HEIGHT],
        value: Hash,
    ) -> Self {

        if next_index == 0 {
            Self {
                next_index: 0.into(),
                levels: core::array::from_fn(|i| MerkleAppendTreeLevel {
                    left: zero_hashes[i],
                    right: zero_hashes[i],
                    zero_hash: zero_hashes[i],
                }),
            }
        }else{
            let mut levels = core::array::from_fn(|i| MerkleAppendTreeLevel {
                left: zero_hashes[i],
                right: zero_hashes[i],
                zero_hash: zero_hashes[i],
            });

            let mut current = value;
            let mut current_index = next_index-1;
            for i in 0..HEIGHT {
                let sibling = siblings[i];
                let swap = (current_index & 1) == 1;
                let new_v = Hasher::two_to_one_swap(swap, &current, &sibling);

                if swap {
                    levels[i].left = sibling;
                    levels[i].right = current;
                    
                }else{
                    levels[i].left = current;
                    levels[i].right = sibling;
                }

                current = new_v;
                
                current_index >>= 1;
            }
            Self {
                next_index: next_index.into(),
                levels,
            }

        }
    }
    pub fn get_next_index(&self) -> u64 {
        self.next_index.into()
    }
    pub fn get_height(&self) -> u8 {
        HEIGHT as u8
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
            let mut siblings = Vec::with_capacity(HEIGHT);
            let value = self.get_value();
            let index: u64 = self.next_index.into();
            let index = index - 1;
            let mut current_index: u64 = index;

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
            let mut siblings = Vec::with_capacity(HEIGHT);
            let value = self.get_value();
            let index: u64 = self.get_next_index()- 1;
            let mut current_index: u64 = index;

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

    pub fn revert_to_index<Hasher: MerkleHasher<Hash>>(&mut self, index: u64, changed_left_siblings: &[Hash], value: Hash) -> QDogeResult<()> {
        if self.next_index == 0 || index >= (self.get_next_index()-1) {
            return Err(DogeBridgeError::RevertIndexTooHigh);
            //anyhow::bail!("Cannot revert to index greater than or equal to current index");
        }

        let mut current_index = (self.next_index)-1;
        let mut revert_index = index;

        let mut next_changed_left_sibling_index = 0;

        let mut current_hash = value;
        let mut i = 0;

        while i < HEIGHT && current_index != revert_index {
            let is_right_child = (revert_index & 1) == 1;
            if is_right_child {
                if next_changed_left_sibling_index == changed_left_siblings.len() {
                    return Err(DogeBridgeError::NotEnoughChangedLeftSiblings);
                }
                self.levels[i].left = changed_left_siblings[next_changed_left_sibling_index];
                self.levels[i].right = current_hash;
                next_changed_left_sibling_index += 1;
            }else{
                self.levels[i].left = current_hash;
                self.levels[i].right = self.levels[i].zero_hash;
                //self.levels[i].left = self.levels[i].zero_hash;
            }
            current_hash = self.levels[i].get_hash::<Hasher>();
            current_index >>= 1;
            revert_index >>= 1;
            i += 1;
        }

        if current_index != revert_index {
            return Err(DogeBridgeError::RevertIndexNotPrefix);
        }
        if next_changed_left_sibling_index != changed_left_siblings.len() {
            return Err(DogeBridgeError::TooManyChangedLeftSiblings);
        }

        while i < HEIGHT {
            let is_right_child = (revert_index & 1) == 1;
            if is_right_child {
                self.levels[i].right = current_hash;
            }else{
                self.levels[i].left = current_hash;
                self.levels[i].right = self.levels[i].zero_hash;
            }
            current_hash = self.levels[i].get_hash::<Hasher>();
            revert_index >>= 1;
            i += 1;
        }



        Ok(())
        
    }

}