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

use doge_light_client::{
    doge::{
        address::{BTCAddress160, BTCAddressType},
        transaction::BTCTransactionOutput,
    },
    hash::{ripemd160::QBTCHash160Hasher, traits::BytesHasher},
};

const OP_PUSHBYTES_32: u8 = 0x20;
const OP_DROP: u8 = 117;
const OP_DUP: u8 = 118;
const OP_EQUALVERIFY: u8 = 136;
const OP_HASH160: u8 = 169;
const OP_CHECKSIG: u8 = 172;
const OP_PUSHBYTES_20: u8 = 0x14;

//  size = 1 + 32 + 4 + 20 + 2 = 59
pub const STANDARD_TRANSFER_WITH_MESSAGE_TEMPLATE: [u8; 59] = qed_doge_macros::const_concat_arrays!(
    [OP_PUSHBYTES_32],
    [0; 32], // 1..33
    [OP_DROP, OP_DUP, OP_HASH160, OP_PUSHBYTES_20],
    [0; 20], // 37..57
    [OP_EQUALVERIFY, OP_CHECKSIG]
);

pub fn get_transfer_with_message_redeem_script(
    message: &[u8],
    public_key_hash: &[u8],
) -> [u8; 59] {
    let mut base = STANDARD_TRANSFER_WITH_MESSAGE_TEMPLATE.clone();
    base[1..33].copy_from_slice(message);
    base[37..57].copy_from_slice(public_key_hash);

    base
}

pub fn get_bridge_deposit_address_hash_v1(
    user_public_key: &[u8],
    bridge_public_key_hash: &[u8],
) -> [u8; 20] {
    QBTCHash160Hasher::hash_bytes(&get_transfer_with_message_redeem_script(
        user_public_key,
        bridge_public_key_hash,
    ))
}

pub fn get_bridge_deposit_address_v1(
    user_public_key: &[u8],
    bridge_public_key_hash: &[u8],
) -> BTCAddress160 {
    BTCAddress160 {
        address_type: BTCAddressType::P2SH,
        address: get_bridge_deposit_address_hash_v1(user_public_key, bridge_public_key_hash),
    }
}

pub fn is_bridge_desposit_output_v1_for_user(
    output: &BTCTransactionOutput,
    user_public_key: &[u8],
    bridge_public_key_hash: &[u8],
) -> bool {
    if output.is_p2sh_output() {
        let output_addr: [u8; 20] = output.script[2..22].try_into().unwrap();
        get_bridge_deposit_address_hash_v1(user_public_key, bridge_public_key_hash) == output_addr
    } else {
        false
    }
}
