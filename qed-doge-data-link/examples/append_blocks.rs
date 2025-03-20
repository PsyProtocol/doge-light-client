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

use qed_doge_data_link::{
    block_header_cache::BlockHeaderFetcher, bridge_state_helpers::gen_bridge_initial_state,
    electrs_link::DogeLinkElectrsClient, traits::QDogeBlockHeaderFetcher,
};
use doge_light_client::{constants::DogeMainNetConfig, network_params::DogeNetworkType};

const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize = 4;
const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize = 32;
const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize = 32;

fn run_gen_block_start(
    mut fetcher: BlockHeaderFetcher,
    start_tip: u32,
    block_count: u32,
) -> anyhow::Result<()> {
    let mut tracker = gen_bridge_initial_state::<
        _,
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >(&mut fetcher, start_tip)?;

    assert_eq!(start_tip, tracker.get_tip_block_number());

    for i in 0..block_count {
        let new_tip = start_tip + i + 1;
        let block_header = fetcher.get_qdoge_block_header(new_tip)?;
        tracker.append_block::<DogeMainNetConfig>(new_tip, &block_header, None)?;
        println!("new_tip: {}", tracker.get_tip_block_number());
    }

    Ok(())
}

fn main() {
    let mut fetcher = BlockHeaderFetcher::new(DogeLinkElectrsClient::new(
        "https://doge-electrs-demo.qed.me".to_string(),
        DogeNetworkType::MainNet,
    ));

    fetcher
        .load_block_headers_bin("test_data/mainnet_headers_5610330-5611352.bin")
        .unwrap();

    let start_block = 5610352 + 32;
    run_gen_block_start(fetcher, start_block, 500).unwrap();
}
