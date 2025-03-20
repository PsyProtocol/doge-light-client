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

use doge_light_client::{chain_state::QEDDogeChainStateCore, init_params::InitBlockDataIBC};

use crate::traits::QDogeBlockHeaderFetcher;
use zerocopy::IntoBytes;

pub fn gen_bridge_initial_state<
    HF: QDogeBlockHeaderFetcher,
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize,
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
>(
    fetcher: &mut HF,
    new_tip: u32,
) -> anyhow::Result<
    QEDDogeChainStateCore<
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >,
> {
    if new_tip < QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE as u32 {
        return Err(anyhow::anyhow!(
            "new_tip must be greater or equal to than {}",
            QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE
        ));
    }
    let start_block = new_tip - (QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE - 1) as u32;

    let base_blocks =
        fetcher.get_qdoge_block_headers_cache(&(start_block..=new_tip).collect::<Vec<u32>>())?;

    let init_data = InitBlockDataIBC::<
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >::new_from_block_headers_empty_tree(
        &base_blocks.try_into().unwrap(), new_tip
    );

    Ok(QEDDogeChainStateCore::<
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >::from_init_data(&init_data))
}

pub fn gen_bridge_initial_state_data<
    HF: QDogeBlockHeaderFetcher,
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize,
    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize,
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize,
>(
    fetcher: &mut HF,
    new_tip: u32,
) -> anyhow::Result<Vec<u8>> {
    let state_data = gen_bridge_initial_state::<
        HF,
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >(fetcher, new_tip)?;
    let state_data_bytes = state_data.as_bytes().to_vec().clone();
    Ok(state_data_bytes)
}
