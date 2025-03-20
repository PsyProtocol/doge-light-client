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

use super::{delta_merkle_proof::DeltaMerkleProofCore, utils::compute_root_merkle_proof_generic};


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MerkleProofCore<Hash: PartialEq + Copy> {
    pub root: Hash,
    pub value: Hash,

    pub index: u64,
    pub siblings: Vec<Hash>,
}




#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MerkleProofCorePartial<Hash: PartialEq + Copy> {
    pub value: Hash,
    pub index: u64,
    pub siblings: Vec<Hash>,
}

impl<Hash: PartialEq + Copy + Default> Default for MerkleProofCore<Hash> {
    fn default() -> Self {
        Self {
            root: Default::default(),
            value: Default::default(),
            index: Default::default(),
            siblings: Default::default(),
        }
    }
}
impl<Hash: PartialEq + Copy> MerkleProofCore<Hash> {
    pub fn new_from_params<Hasher: MerkleHasher<Hash>>(
        index: u64,
        value: Hash,
        siblings: Vec<Hash>,
    ) -> Self {
        let root = compute_root_merkle_proof_generic::<Hash, Hasher>(value, index, &siblings);
        Self {
            root,
            value,
            index,
            siblings,
        }
    }
    pub fn verify<Hasher: MerkleHasher<Hash>>(&self) -> bool {
        compute_root_merkle_proof_generic::<Hash, Hasher>(self.value, self.index, &self.siblings)
            == self.root
    }
    pub fn verify_btc_block_tx_tree<Hasher: MerkleHasher<Hash>>(&self) -> bool {
        let mut current = self.value;
        for (i, sibling) in self.siblings.iter().enumerate() {
            if self.index & (1 << i) == 0 {
                current = Hasher::two_to_one(&current, sibling);
            } else {
                if sibling.eq(&current) {
                    // if the current path is on the right and the left sibling is the same, then the current path is not part of the valid tree span
                    return false;
                }
                current = Hasher::two_to_one(sibling, &current);
            }
        }
        current == self.root
    }
    pub fn into_delta_merkle_proof(self) -> DeltaMerkleProofCore<Hash> {
        DeltaMerkleProofCore {
            old_root: self.root,
            new_root: self.root,
            old_value: self.value,
            new_value: self.value,
            index: self.index,
            siblings: self.siblings,
        }
    }
    pub fn to_delta_merkle_proof(&self) -> DeltaMerkleProofCore<Hash> {
        DeltaMerkleProofCore {
            old_root: self.root,
            new_root: self.root,
            old_value: self.value,
            new_value: self.value,
            index: self.index,
            siblings: self.siblings.clone(),
        }
    }
}



impl<Hash: PartialEq + Copy + Default> Default for MerkleProofCorePartial<Hash> {
    fn default() -> Self {
        Self {
            value: Default::default(),
            index: Default::default(),
            siblings: Default::default(),
        }
    }
}


impl<Hash: PartialEq + Copy> MerkleProofCorePartial<Hash> {
    pub fn new_from_params(
        index: u64,
        value: Hash,
        siblings: Vec<Hash>,
    ) -> Self {
        Self {
            value,
            index,
            siblings,
        }
    }
    pub fn to_full<Hasher: MerkleHasher<Hash>>(&self) -> MerkleProofCore<Hash> {
        let root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.value, self.index, &self.siblings);
        MerkleProofCore {
            root,
            value: self.value,
            index: self.index,
            siblings: self.siblings.clone(),
        }
    }
    pub fn into_full<Hasher: MerkleHasher<Hash>>(self) -> MerkleProofCore<Hash> {
        let root = compute_root_merkle_proof_generic::<Hash, Hasher>(self.value, self.index, &self.siblings);
        MerkleProofCore {
            root,
            value: self.value,
            index: self.index,
            siblings: self.siblings,
        }
    }
}

impl<Hash: PartialEq + Copy> From<MerkleProofCore<Hash>> for MerkleProofCorePartial<Hash> {
    fn from(value: MerkleProofCore<Hash>) -> Self {
        Self {
            value: value.value,
            index: value.index,
            siblings: value.siblings,
        }
    }
}
impl<Hash: PartialEq + Copy> From<&MerkleProofCore<Hash>> for MerkleProofCorePartial<Hash> {
    fn from(value: &MerkleProofCore<Hash>) -> Self {
        Self {
            value: value.value,
            index: value.index,
            siblings: value.siblings.clone(),
        }
    }
}