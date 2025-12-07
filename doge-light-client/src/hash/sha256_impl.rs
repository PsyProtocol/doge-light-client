/*
Copyright (C) 2025 Zero Knowledge Labs Limited, Psy Protocol

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

"This software was created by Psy Protocol (https://psy.xyz)
with contributions from Carter Feldman (https://x.com/cmpeq)."
*/

#[cfg(all(not(feature = "solprogram"),feature = "sha2", not(feature = "sp1")))]
use sha2::{Digest, Sha256};
#[cfg(feature = "sp1")]
use sha2_v0_10_9::Sha256;

use crate::common_types::QHash256;

#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_bytes(bytes: &[u8]) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.into()
}

#[cfg(all(feature = "solprogram", not(any(feature = "sha2", feature= "sp1"))))]
#[inline]
pub fn hash_impl_sha256_bytes(bytes: &[u8]) -> QHash256 {
    solana_program::hash::hash(bytes).to_bytes()
}



#[cfg(all(feature = "solprogram", not(any(feature = "sha2", feature= "sp1"))))]
#[inline]
pub fn hash_impl_sha256_two_to_one_bytes_buf(buf: &mut [u8; 64], left: &QHash256, right: &QHash256) -> QHash256 {
    buf[..32].copy_from_slice(left.as_ref());
    buf[32..].copy_from_slice(right.as_ref());
    hash_impl_sha256_bytes(buf)
}

#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_two_to_one_bytes_buf(buf: &mut [u8; 64], left: &QHash256, right: &QHash256) -> QHash256 {
    buf[..32].copy_from_slice(left.as_ref());
    buf[32..].copy_from_slice(right.as_ref());
    hash_impl_sha256_bytes(buf)
}
#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_two_to_one_bytes(left: &QHash256, right: &QHash256) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    result.into()
}
#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_btc_hash256_two_to_one_bytes(left: &QHash256, right: &QHash256) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    Sha256::digest(&result).into()

}
#[cfg(all(feature = "solprogram", not(any(feature = "sha2", feature= "sp1"))))]
#[inline]
pub fn hash_impl_sha256_two_to_one_bytes(left: &QHash256, right: &QHash256) -> QHash256 {
    let mut buf = [0u8; 64];
    buf[..32].copy_from_slice(left.as_ref());
    buf[32..].copy_from_slice(right.as_ref());
    hash_impl_sha256_bytes(&buf)
}


#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_hash_two_buffers_concat(left: &[u8], right: &[u8]) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    let result = hasher.finalize();
    result.into()
}


#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_hash_three_buffers_concat(a: &[u8], b: &[u8], c: &[u8]) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    hasher.update(c);
    let result = hasher.finalize();
    result.into()
}
#[cfg(all(not(feature = "solprogram"),any(feature = "sha2", feature= "sp1")))]
#[inline]
pub fn hash_impl_sha256_hash_four_buffers_concat(a: &[u8], b: &[u8], c: &[u8], d: &[u8]) -> QHash256 {
    let mut hasher = Sha256::new();
    hasher.update(a);
    hasher.update(b);
    hasher.update(c);
    hasher.update(d);
    let result = hasher.finalize();
    result.into()
}