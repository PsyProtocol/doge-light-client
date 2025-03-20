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

use std::collections::HashSet;

use qed_doge_data_link::{block_cache::BlockFetcher, electrs_link::DogeLinkElectrsClient};
use doge_light_client::{doge::{address::BTCAddress160, transaction::BTCTransaction}, network_params::DogeNetworkType};
fn get_doge_output_addresses_for_tx(tx: &BTCTransaction) -> Vec<BTCAddress160> {
    let mut addrs = tx.outputs.iter().map(|x| x.get_output_address()).filter(|x|x.is_ok()).map(|x|x.unwrap()).collect::<Vec<BTCAddress160>>();
    addrs.sort();
    addrs.dedup();
    addrs
}
fn run_cache_blocks(
    start_block: u32,
    total: usize,
    client: DogeLinkElectrsClient,
    file_path: &str,
) -> anyhow::Result<()> {
    let end_block = start_block + total as u32;

    let mut cache = BlockFetcher::new(client);
    cache.load_blocks_bin(file_path)?;

    let binding = (start_block..end_block).collect::<Vec<u32>>();
    let chunks = binding.chunks(10);
    

    
    let mut output_addresses = HashSet::new();
    for c in chunks {
        let blocks = cache.get_blocks(c)?;
        for block in blocks.iter() {
            for tx in block.transactions.iter() {
                let addrs = get_doge_output_addresses_for_tx(tx);
                for addr in addrs.iter() {
                    if output_addresses.insert(addr.to_owned()) {
                        println!("new address: {}", addr.to_address_string());
                    }else{
                        println!("old address: {}", addr.to_address_string());

                    }
                }
            }
        }
        //cache.save_blocks_bin(file_path)?;
        println!("processed blocks {} -> {}", c[0], c[c.len() - 1]);
    }
    Ok(())
}

fn main() {
    std::fs::create_dir_all("test_blocks").unwrap();


    run_cache_blocks(
        7655400,
        7655839-7655400,
        DogeLinkElectrsClient::new(
            "https://doge-electrs-testnet-demo.qed.me".to_string(),
            DogeNetworkType::TestNet,
        ),
        format!("test_blocks/testnet_blocks_7655400-7655839.bin").as_str(),
    )
    .unwrap();
}
