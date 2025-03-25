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

use crate::network_params::{DogeNetworkParams, DogeNetworkType};
/*
// P2SH: regtest = 0xc4, testnet = 0xc4, mainnet = 0x16
pub const P2SH_ADDRESS_CHECK58_VERSION: u8 = 0xc4;

// P2PKH: regtest = 0x6f, testnet = 0x71, mainnet = 0x1e
pub const P2PKH_ADDRESS_CHECK58_VERSION: u8 = 0x71;


pub const DOGE_NETWORK_TYPE: DogeNetworkType = DogeNetworkType::MainNet;

*/
pub const MERGED_MINING_HEADER: [u8; 4] = [0xfa, 0xbe, 0x6d, 0x6d];

const DOGE_REGTEST_PARAMS: DogeNetworkParams = DogeNetworkParams {
    allow_min_difficulty_blocks: true,
    pow_target_timespan: 1,
    pow_target_spacing: 1,
    pow_limit: 545259519,
    strict_chain_id: true,
    aux_pow_chain_id: 0x0062,
    min_valid_height: 22,
};
const DOGE_TESTNET_PARAMS: DogeNetworkParams = DogeNetworkParams {
    allow_min_difficulty_blocks: true,
    pow_target_timespan: 60,
    pow_target_spacing: 60,
    pow_limit: 504365055,
    strict_chain_id: false,
    aux_pow_chain_id: 0x0062,
    min_valid_height: 158102,
};
const DOGE_MAINNET_PARAMS: DogeNetworkParams = DogeNetworkParams {
    allow_min_difficulty_blocks: false,
    pow_target_timespan: 60,
    pow_target_spacing: 60,
    pow_limit: 504365055,
    aux_pow_chain_id: 0x0062,
    strict_chain_id: true,
    min_valid_height: 371339,

};

pub trait DogeNetworkConfig {
    const NETWORK_TYPE: DogeNetworkType;
    const NETWORK_PARAMS: DogeNetworkParams;

    const P2PKH_VERSION_BYTE: u8;
    const P2SH_VERSION_BYTE: u8;
    const PRIVATE_KEY_VERSION_BYTE: u8;
    const START_ADDRESS_STRING_CHAR: char;
    const START_ADDRESS_STRING_BYTE: u8;
}

#[derive(Clone, Copy, Default, Ord, PartialEq, Eq, PartialOrd)]
pub struct DogeMainNetConfig;
impl DogeNetworkConfig for DogeMainNetConfig {
    const NETWORK_TYPE: DogeNetworkType = DogeNetworkType::MainNet;
    const NETWORK_PARAMS: DogeNetworkParams = DOGE_MAINNET_PARAMS;

    const P2PKH_VERSION_BYTE: u8 = 0x1E;
    const P2SH_VERSION_BYTE: u8 = 0x16;
    const PRIVATE_KEY_VERSION_BYTE: u8 = 0x9E;
    const START_ADDRESS_STRING_CHAR: char = 'D';
    const START_ADDRESS_STRING_BYTE: u8 = 0x44;
}

#[derive(Clone, Copy, Default, Ord, PartialEq, Eq, PartialOrd)]
pub struct DogeTestNetConfig;
impl DogeNetworkConfig for DogeTestNetConfig {
    const NETWORK_TYPE: DogeNetworkType = DogeNetworkType::TestNet;
    const NETWORK_PARAMS: DogeNetworkParams = DOGE_TESTNET_PARAMS;

    const P2PKH_VERSION_BYTE: u8 = 0x71;
    const P2SH_VERSION_BYTE: u8 = 0xC4;
    const PRIVATE_KEY_VERSION_BYTE: u8 = 0xF1;
    const START_ADDRESS_STRING_CHAR: char = 'n';
    const START_ADDRESS_STRING_BYTE: u8 = 0x6E;
}

#[derive(Clone, Copy, Default, Ord, PartialEq, Eq, PartialOrd)]
pub struct DogeRegTestConfig;
impl DogeNetworkConfig for DogeRegTestConfig {
    const NETWORK_TYPE: DogeNetworkType = DogeNetworkType::RegTest;
    const NETWORK_PARAMS: DogeNetworkParams = DOGE_REGTEST_PARAMS;

    const P2PKH_VERSION_BYTE: u8 = 0x6F;
    const P2SH_VERSION_BYTE: u8 = 0xC4;
    const PRIVATE_KEY_VERSION_BYTE: u8 = 0xEF;
    const START_ADDRESS_STRING_CHAR: char = 'm';
    const START_ADDRESS_STRING_BYTE: u8 = 0x6D;
}


//pub const DOGE_NETWORK_PARAMS: DogeNetworkParams = DOGE_MAINNET_PARAMS;

pub const VERSION_AUXPOW: u32 = 1 << 8;
