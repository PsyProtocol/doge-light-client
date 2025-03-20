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

use crate::core_data::QHash256;

use super::{sha256_impl::hash_impl_sha256_bytes, traits::{iterate_merkle_hasher, BytesHasher, MerkleHasher, MerkleZeroHasher}};



#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct QSha256Hasher;


impl BytesHasher<QHash256> for QSha256Hasher {
    fn hash_bytes(data: &[u8]) -> QHash256 {
        hash_impl_sha256_bytes(data)
    }
}
impl MerkleHasher<QHash256> for QSha256Hasher {
    fn two_to_one(left: &QHash256, right: &QHash256) -> QHash256 {
        let mut bytes = [0u8; 64];
        bytes[0..32].copy_from_slice(left);
        bytes[32..64].copy_from_slice(right);
        hash_impl_sha256_bytes(&bytes)
    }
}
impl MerkleZeroHasher<QHash256> for QSha256Hasher {
    fn get_zero_hash(reverse_level: usize) -> QHash256 {
        iterate_merkle_hasher::<QHash256, Self>([0u8;32], reverse_level)
    }
}




#[derive(Clone, Copy)]
pub struct QBTCHash256Hasher;


impl BytesHasher<QHash256> for QBTCHash256Hasher {
    fn hash_bytes(data: &[u8]) -> QHash256 {
        hash_impl_sha256_bytes(&hash_impl_sha256_bytes(data))
    }
}
impl MerkleHasher<QHash256> for QBTCHash256Hasher {
    fn two_to_one(left: &QHash256, right: &QHash256) -> QHash256 {
        let mut bytes = [0u8; 64];
        bytes[0..32].copy_from_slice(left);
        bytes[32..64].copy_from_slice(right);
        hash_impl_sha256_bytes(&hash_impl_sha256_bytes(&bytes))
    }
}
impl MerkleZeroHasher<QHash256> for QBTCHash256Hasher {
    fn get_zero_hash(reverse_level: usize) -> QHash256 {
        iterate_merkle_hasher::<QHash256, Self>([0u8;32], reverse_level)
    }
}