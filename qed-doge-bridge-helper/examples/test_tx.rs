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

use std::str::FromStr;

use doge_light_client::{constants::DogeTestNetConfig, doge::address::BTCAddress160};
use qed_doge_bridge_helper::tx_template::get_bridge_deposit_address_v1;

fn main(){
    let solana_public_key = hex_literal::hex!("e83c24b97aeadd8de838b7c040347ac9e821a103c38b2999a7989f7a6181e0d8");

    let bridge_public_key_hash = BTCAddress160::from_str("nidKRv4eeRaLzngA34r8epXFNnJS54GJ1R").unwrap().address;
    println!("bridgepkh: {}", hex::encode(bridge_public_key_hash));

    let deposit_address = get_bridge_deposit_address_v1(&solana_public_key, &bridge_public_key_hash);

    println!("deposit_address: {}", deposit_address.to_address_string::<DogeTestNetConfig>());





}