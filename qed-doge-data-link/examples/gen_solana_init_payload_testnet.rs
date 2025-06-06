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

use qed_doge_data_link::{block_header_cache::BlockHeaderFetcher, bridge_state_helpers::gen_bridge_initial_state_data, electrs_link::DogeLinkElectrsClient};
use doge_light_client::network_params::DogeNetworkType;

fn main() {

    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize = 4;
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize = 32;
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize = 32;

    let mut fetcher = BlockHeaderFetcher::new(DogeLinkElectrsClient::new(
        "https://doge-electrs-testnet-demo.qed.me".to_string(),
        DogeNetworkType::TestNet,
    ));

    //fetcher.load_block_headers_bin("test_data/testnet_block_headers_7654400-7654500.bin").unwrap();
    
    let new_tip = 7667430;
    let data = gen_bridge_initial_state_data::<_, QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE, QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS, QDOGE_BRIDGE_BLOCK_TREE_HEIGHT>(
        &mut fetcher,
        new_tip
    ).expect("error generating initial state data");

    println!("tip: {}", new_tip);
    println!("data: {}",hex::encode(&data));

}
