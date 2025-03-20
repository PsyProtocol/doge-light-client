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

use crate::{
    constants::DogeNetworkConfig, core_data::QHash256, math::btc_difficulty::BTCDifficulty
};

fn allow_min_difficulty_for_block<NC: DogeNetworkConfig>(current_block_time: i64, last_block_time: i64) -> bool {
    NC::NETWORK_PARAMS.allow_min_difficulty_blocks
        && current_block_time > (last_block_time + NC::NETWORK_PARAMS.pow_target_spacing * 2)
}

pub fn get_next_work_required<NC: DogeNetworkConfig>(
    last_height: u32,
    last_block_time: i64,
    last_bits: u32,
    first_block_time: i64,
    current_block_time: i64,
) -> u32 {
    if allow_min_difficulty_for_block::<NC>(current_block_time, last_block_time) {
        NC::NETWORK_PARAMS.pow_limit
    } else {
        calc_dogecoin_next_work_required_full(
            last_height,
            last_block_time,
            last_bits,
            first_block_time,
            NC::NETWORK_PARAMS.pow_target_timespan,
            &BTCDifficulty::new_from_bits(NC::NETWORK_PARAMS.pow_limit),
            true
        )
    }
}
pub fn calc_dogecoin_next_work_required_full(
    last_height: u32,
    last_block_time: i64,
    last_bits: u32,
    first_block_time: i64,
    pow_target_timespan: i64,
    pow_limit: &BTCDifficulty,
    f_digishield_difficulty_calculation: bool,
) -> u32 {
    
    let actual_timespan = last_block_time - first_block_time;
    let mut modulated_timespan = actual_timespan;
    let mut min_timespan = pow_target_timespan / 16;
    let mut max_timespan = pow_target_timespan * 4;

    if f_digishield_difficulty_calculation {
        let diff = (modulated_timespan - pow_target_timespan) / 8;
        modulated_timespan = pow_target_timespan + diff;

        min_timespan = pow_target_timespan - (pow_target_timespan / 4);
        max_timespan = pow_target_timespan + (pow_target_timespan / 2);
    } else if last_height > 10000 {
        min_timespan = pow_target_timespan / 4;
        max_timespan = pow_target_timespan * 4;
    } else if last_height > 5000 {
        min_timespan = pow_target_timespan / 8;
        max_timespan = pow_target_timespan * 4;
    } else {
        //min_timespan = pow_target_timespan / 16;
        //max_timespan = pow_target_timespan * 4;
    }

    if modulated_timespan < min_timespan {
        modulated_timespan = min_timespan;
    } else if modulated_timespan > max_timespan {
        modulated_timespan = max_timespan;
    }

    let bn_new = BTCDifficulty::new_from_bits(last_bits);


    let bn_new = bn_new.to_adjust_for_next_work(
        modulated_timespan,
        pow_target_timespan,
    );

    if bn_new.is_gt(pow_limit) {
        pow_limit.to_compact_bits()
    } else {
        bn_new.to_compact_bits()
    }
}

pub fn check_proof_of_work<NC: DogeNetworkConfig>(pow_hash: QHash256, n_bits: u32) -> bool {
    let difficulty = BTCDifficulty::new_from_bits_0_if_overflow(n_bits);
    
    let mut reversed_pow_hash = pow_hash.clone();
    reversed_pow_hash.reverse();

    let pow_hash_dif = BTCDifficulty::new_from_hash(reversed_pow_hash);


    if difficulty.is_zero()
        || difficulty.is_gt(&BTCDifficulty::new_from_bits(
            NC::NETWORK_PARAMS.pow_limit,
        ))
    {
        false
    } else {
        pow_hash_dif.is_leq(&difficulty)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calc_next_work_mainnet_1() {

        /* 
        lastHeight: 145001,
        nFirstBlockTime: 1395094679,
        lastBlockTime: 1395094727,
        lastBits: 0x1b671062,
        expectedNextBits: 0x1b6558a4,*/

        let pow_target_timespan_mainnet = 60;
        let pow_limit_mainnet = BTCDifficulty::new_from_hash(hex_literal::hex!("00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
        let f_digishield_difficulty_calculation = true;

        let last_height = 145001;
        let last_block_time = 1395094727;
        let last_bits = 0x1b671062;
        let first_block_time = 1395094679;
        let expected_next_bits = 0x1b6558a4;
        let computed_next_bits  = calc_dogecoin_next_work_required_full(
            last_height,
            last_block_time,
            last_bits,
            first_block_time,
            pow_target_timespan_mainnet,
            &pow_limit_mainnet,
            f_digishield_difficulty_calculation
        );

        assert_eq!(expected_next_bits, computed_next_bits, "expected_next_bits: {:x}, computed_next_bits: {:x}", expected_next_bits, computed_next_bits);

    }
}
