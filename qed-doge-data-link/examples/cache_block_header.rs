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

use qed_doge_data_link::{block_header_cache::BlockHeaderFetcher, electrs_link::DogeLinkElectrsClient, traits::QDogeBlockHeaderFetcher};
use doge_light_client::network_params::DogeNetworkType;


fn run_cache_block_headers(
    start_block: u32,
    total: usize,
    client: DogeLinkElectrsClient,
    file_path: &str,
) -> anyhow::Result<()> {
    let end_block = start_block + total as u32;

    let mut cache = BlockHeaderFetcher::new(client);

    let binding = (start_block..end_block).collect::<Vec<u32>>();
    let chunks = binding.chunks(10);
    

    
    for c in chunks {
        let _blocks = cache.get_qdoge_block_headers_cache(c)?;
        cache.save_block_headers_bin(file_path)?;
        println!("saved blocks {} -> {}", c[0], c[c.len() - 1]);
    }
    Ok(())
}

fn main() {
    std::fs::create_dir_all("test_blocks").unwrap();

    let start_block = 7654400;
    let count = 100;
    let end_block = start_block+count as u32;

    run_cache_block_headers(
        start_block,
        count,
        DogeLinkElectrsClient::new(
            "https://doge-electrs-testnet-demo.qed.me".to_string(),
            DogeNetworkType::TestNet,
        ),
        format!("test_blocks/testnet_block_headers_{}-{}.bin", start_block, end_block).as_str(),
    )
    .unwrap();
}
