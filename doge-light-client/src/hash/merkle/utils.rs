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

use crate::hash::traits::MerkleHasher;


pub fn compute_root_merkle_proof_generic<Hash: PartialEq + Copy, H: MerkleHasher<Hash>>(
    value: Hash,
    index: u64,
    siblings: &[Hash]
) -> Hash {
    let mut current = value;
    for (i, sibling) in siblings.iter().enumerate() {
        current = H::two_to_one_swap((index & (1 << i)) == 1,&current, sibling);
    }
    current
}


pub fn compute_partial_merkle_root_from_leaves<
    Hash: PartialEq + Copy,
    Hasher: MerkleHasher<Hash>,
>(
    leaves: &[Hash],
) -> Hash {
    let mut current = leaves.to_vec();
    while current.len() > 1 {
        let mut next = vec![];
        for i in 0..current.len() / 2 {
            next.push(Hasher::two_to_one(&current[2 * i], &current[2 * i + 1]));
        }
        if current.len() % 2 == 1 {
            next.push(current[current.len() - 1]);
        }
        current = next;
    }
    current[0]
}