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
    constants::DogeNetworkConfig, core_data::{QDogeBlockHeader, QHash256}, error::{DogeBridgeError, QDogeResult}, network_params::DogeNetworkType
};

use super::check_doge_block_seq::{check_proof_of_work, get_next_work_required};

pub fn check_block_header_err<NC: DogeNetworkConfig>(
    last_height: u32,
    block_header: &QDogeBlockHeader,
    last_block_time: u32,
    last_bits: u32,
    first_block_time: u32,
    known_pow_block_hash: Option<QHash256>,
) -> QDogeResult<()> {
    if block_header.header.is_aux_pow() != block_header.aux_pow.is_some() {
        return Err(DogeBridgeError::AuxPowVersionBitsMismatch);
    }
    if NC::NETWORK_PARAMS.strict_chain_id
        && NC::NETWORK_PARAMS.aux_pow_chain_id != block_header.header.get_chain_id()
    {
        return Err(DogeBridgeError::AuxPowChainIdMismatch);
    }
    let expected_difficulty_bits = get_next_work_required::<NC>(
        last_height,
        last_block_time as i64,
        last_bits,
        first_block_time as i64,
        block_header.header.timestamp as i64,
    );
    if expected_difficulty_bits != block_header.header.bits {
        return Err(DogeBridgeError::DifficutlyBitsMismatch);
    }
    if block_header.aux_pow.is_none() {
        if !check_proof_of_work::<NC>(if known_pow_block_hash.is_some() {
            known_pow_block_hash.unwrap()
        } else {
            block_header.header.get_pow_hash()
        }, block_header.header.bits) {
            return Err(DogeBridgeError::StandardPoWCheckFailed);
        }
    } else {
        if !check_proof_of_work::<NC>(
            if known_pow_block_hash.is_some() {
                known_pow_block_hash.unwrap()
            } else {
                block_header
                    .aux_pow
                    .as_ref()
                    .unwrap()
                    .parent_block
                    .get_pow_hash()
            },
            block_header.header.bits,
        ) {
            return Err(DogeBridgeError::AuxPowParentBlockPoWCheckFailed);
        }
        block_header.aux_pow.as_ref().unwrap().check_err::<NC>(
            block_header.header.get_hash(),
            block_header.header.get_chain_id(),
        )?;
    }
    Ok(())
}
