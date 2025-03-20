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

use crate::hash::traits::MerkleHasher;

use super::utils::compute_root_merkle_proof_generic;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeltaMerkleProofCore<Hash: PartialEq + Copy> {
    pub old_root: Hash,
    pub old_value: Hash,

    pub new_root: Hash,
    pub new_value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DeltaMerkleProofCorePartial<Hash: PartialEq + Copy> {
    pub old_value: Hash,
    pub new_value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}

impl<Hash: PartialEq + Copy> DeltaMerkleProofCore<Hash> {
    pub fn from_params<H: MerkleHasher<Hash>>(
        index: u64,
        old_value: Hash,
        new_value: Hash,
        siblings: Vec<Hash>,
    ) -> Self {
        let old_root = compute_root_merkle_proof_generic::<Hash, H>(old_value, index, &siblings);
        let new_root = compute_root_merkle_proof_generic::<Hash, H>(new_value, index, &siblings);

        Self {
            old_root,
            old_value,
            new_root,
            new_value,
            index,
            siblings,
        }
    }
    pub fn single_value(index: u64, old_value: Hash, new_value: Hash) -> Self {
        Self {
            old_root: old_value,
            old_value,
            new_root: new_value,
            new_value,
            index,
            siblings: Vec::new(),
        }
    }
    pub fn with_shortened_height_from_bottom<H: MerkleHasher<Hash>>(
        &self,
        new_height: usize,
    ) -> Self {
        assert!(
            new_height <= self.siblings.len(),
            "cannot shorten tree to a height taller than the current proof"
        );
        if new_height == self.siblings.len() {
            self.clone()
        } else {
            let height_diff = self.siblings.len() - new_height;
            let low_index = self.index & ((1u64 << (height_diff as u64)) - 1u64);
            let new_index = self.index >> (height_diff as u64);
            let old_value = compute_root_merkle_proof_generic::<Hash, H>(
                self.old_value,
                low_index,
                &self.siblings[0..height_diff],
            );
            let new_value = compute_root_merkle_proof_generic::<Hash, H>(
                self.new_value,
                low_index,
                &self.siblings[0..height_diff],
            );

            Self::from_params::<H>(
                new_index,
                old_value,
                new_value,
                self.siblings[height_diff..].to_vec(),
            )
        }
    }
    pub fn shorten_height<H: MerkleHasher<Hash>>(&self, new_height: usize) -> Self {
        assert!(
            new_height <= self.siblings.len(),
            "cannot shorten tree to a height taller than the current proof"
        );
        if new_height == self.siblings.len() {
            self.clone()
        } else {
            Self::from_params::<H>(
                self.index,
                self.old_value,
                self.new_value,
                self.siblings[0..new_height].to_vec(),
            )
        }
    }
    pub fn verify<Hasher: MerkleHasher<Hash>>(&self) -> bool {
        compute_root_merkle_proof_generic::<Hash, Hasher>(
            self.old_value,
            self.index,
            &self.siblings,
        ) == self.old_root
            && compute_root_merkle_proof_generic::<Hash, Hasher>(
                self.new_value,
                self.index,
                &self.siblings,
            ) == self.new_root
    }
}

impl<Hash: PartialEq + Copy + Default> Default for DeltaMerkleProofCore<Hash> {
    fn default() -> Self {
        Self {
            old_root: Default::default(),
            old_value: Default::default(),
            new_root: Default::default(),
            new_value: Default::default(),
            index: Default::default(),
            siblings: Default::default(),
        }
    }
}


impl<Hash: PartialEq + Copy + Default> Default for DeltaMerkleProofCorePartial<Hash> {
    fn default() -> Self {
        Self {
            old_value: Default::default(),
            new_value: Default::default(),
            index: Default::default(),
            siblings: Default::default(),
        }
    }
}



impl<Hash: PartialEq + Copy> DeltaMerkleProofCorePartial<Hash> {
    pub fn new_from_params(
        index: u64,
        old_value: Hash,
        new_value: Hash,
        siblings: Vec<Hash>,
    ) -> Self {
        Self {
            old_value,
            new_value,
            index,
            siblings,
        }
    }
    pub fn to_full<Hasher: MerkleHasher<Hash>>(&self) -> DeltaMerkleProofCore<Hash> {
        let old_root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.old_value, self.index, &self.siblings);
        let new_root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.new_value, self.index, &self.siblings);
        DeltaMerkleProofCore {
            old_root,
            old_value: self.old_value,
            new_root,
            new_value: self.new_value,
            index: self.index,
            siblings: self.siblings.clone(),
        }
    }
    pub fn into_full<Hasher: MerkleHasher<Hash>>(self) -> DeltaMerkleProofCore<Hash> {
        let old_root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.old_value, self.index, &self.siblings);
        let new_root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.new_value, self.index, &self.siblings);
        DeltaMerkleProofCore {
            old_root,
            old_value: self.old_value,
            new_root,
            new_value: self.new_value,
            index: self.index,
            siblings: self.siblings,
        }
    }
}

impl<Hash: PartialEq + Copy> From<DeltaMerkleProofCore<Hash>> for DeltaMerkleProofCorePartial<Hash> {
    fn from(value: DeltaMerkleProofCore<Hash>) -> Self {
        Self {
            old_value: value.old_value,
            new_value: value.new_value,
            index: value.index,
            siblings: value.siblings,
        }
    }
}
impl<Hash: PartialEq + Copy> From<&DeltaMerkleProofCore<Hash>> for DeltaMerkleProofCorePartial<Hash> {
    fn from(value: &DeltaMerkleProofCore<Hash>) -> Self {
        Self {
            old_value: value.old_value,
            new_value: value.new_value,
            index: value.index,
            siblings: value.siblings.clone(),
        }
    }
}