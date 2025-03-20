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

use crate::core_data::QHash160;

use super::{
    ripemd160_impl::hash_impl_ripemd160_bytes, sha256_impl::hash_impl_sha256_bytes, traits::{iterate_merkle_hasher, BytesHasher, MerkleHasher, MerkleZeroHasher}
};

#[derive(Clone, Copy)]
pub struct QRipemd160Hasher;

impl BytesHasher<QHash160> for QRipemd160Hasher {
    fn hash_bytes(data: &[u8]) -> QHash160 {
        hash_impl_ripemd160_bytes(data)
    }
}
impl MerkleHasher<QHash160> for QRipemd160Hasher {
    fn two_to_one(left: &QHash160, right: &QHash160) -> QHash160 {
        let mut bytes = [0u8; 40];
        bytes[0..20].copy_from_slice(left);
        bytes[20..40].copy_from_slice(right);
        hash_impl_ripemd160_bytes(&bytes)
    }
}
impl MerkleZeroHasher<QHash160> for QRipemd160Hasher {
    fn get_zero_hash(reverse_level: usize) -> QHash160 {
        iterate_merkle_hasher::<QHash160, Self>([0u8; 20], reverse_level)
    }
}



#[derive(Clone, Copy)]
pub struct QBTCHash160Hasher;

impl BytesHasher<QHash160> for QBTCHash160Hasher {
    fn hash_bytes(data: &[u8]) -> QHash160 {
        hash_impl_ripemd160_bytes(&hash_impl_sha256_bytes(data))
    }
}
impl MerkleHasher<QHash160> for QBTCHash160Hasher {
    fn two_to_one(left: &QHash160, right: &QHash160) -> QHash160 {
        let mut bytes = [0u8; 40];
        bytes[0..20].copy_from_slice(left);
        bytes[20..40].copy_from_slice(right);
        hash_impl_ripemd160_bytes(&hash_impl_sha256_bytes(&bytes))
    }
}
impl MerkleZeroHasher<QHash160> for QBTCHash160Hasher {
    fn get_zero_hash(reverse_level: usize) -> QHash160 {
        iterate_merkle_hasher::<QHash160, Self>([0u8; 20], reverse_level)
    }
}
