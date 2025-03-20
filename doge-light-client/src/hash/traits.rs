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

pub trait ZeroableHash: Sized + Copy + Clone {
    fn get_zero_value() -> Self;
}
impl<const N: usize> ZeroableHash for [u8; N] {
    fn get_zero_value() -> Self {
        [0; N]
    }
}

pub trait BytesHasher<Hash: PartialEq> {
    fn hash_bytes(data: &[u8]) -> Hash;
}

pub trait MerkleHasher<Hash: PartialEq> {
    fn two_to_one(left: &Hash, right: &Hash) -> Hash;
    fn two_to_one_swap(swap: bool, left: &Hash, right: &Hash) -> Hash {
        if swap {
            Self::two_to_one(right, left)
        }else{
            Self::two_to_one(left, right)
        }
    }
}

pub trait MerkleZeroHasher<Hash: PartialEq>: MerkleHasher<Hash> {
    fn get_zero_hash(reverse_level: usize) -> Hash;
}

pub fn iterate_merkle_hasher<Hash: PartialEq, Hasher: MerkleHasher<Hash>>(
    mut current: Hash,
    reverse_level: usize,
) -> Hash {
    for _ in 0..reverse_level {
        current = Hasher::two_to_one(&current, &current);
    }
    current
}

pub fn get_zero_hashes<Hash: PartialEq + ZeroableHash, Hasher: MerkleHasher<Hash>>(
    count: usize,
) -> Vec<Hash> {
    let mut hashes = Vec::with_capacity(count);
    hashes.push(Hash::get_zero_value());
    for i in 1..count {
        hashes.push(Hasher::two_to_one(&hashes[i - 1], &hashes[i - 1]));
    }
    hashes
}

pub fn get_zero_hashes_sized<Hash: PartialEq + ZeroableHash + Copy, Hasher: MerkleHasher<Hash>, const N: usize>() -> [Hash; N] {
    let v = Hash::get_zero_value();
    let mut hashes = [v; N];
    for i in 1..N {
        hashes[i] = Hasher::two_to_one(&hashes[i - 1], &hashes[i - 1]);
    }
    hashes
}


pub const ZERO_HASH_CACHE_SIZE: usize = 128;
pub trait MerkleZeroHasherWithCache<Hash: PartialEq + Copy>: MerkleHasher<Hash> {
    const CACHED_ZERO_HASHES: [Hash; ZERO_HASH_CACHE_SIZE];
}
impl<Hash: PartialEq + Copy, T: MerkleZeroHasherWithCache<Hash>> MerkleZeroHasher<Hash> for T {
    fn get_zero_hash(reverse_level: usize) -> Hash {
        if reverse_level < ZERO_HASH_CACHE_SIZE {
            T::CACHED_ZERO_HASHES[reverse_level]
        } else {
            let current = T::CACHED_ZERO_HASHES[ZERO_HASH_CACHE_SIZE - 1];
            iterate_merkle_hasher::<Hash, Self>(current, reverse_level - ZERO_HASH_CACHE_SIZE + 1)
        }
    }
}


pub trait QStandardHasher<Hash: PartialEq + Copy>: MerkleHasher<Hash> + BytesHasher<Hash> + MerkleZeroHasher<Hash> {
}

impl<T: MerkleHasher<Hash> + BytesHasher<Hash> + MerkleZeroHasher<Hash>, Hash: PartialEq + Copy> QStandardHasher<Hash> for T {
}