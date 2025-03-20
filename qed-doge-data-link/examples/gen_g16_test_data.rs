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

use qed_doge_data_link::{block_header_cache::BlockHeaderFetcher, electrs_link::DogeLinkElectrsClient};
use doge_light_client::network_params::DogeNetworkType;
use serde::{Deserialize, Serialize};

use qed_doge_data_link::hex_helpers::{hex_array_32, hex_array_80};

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct DogeScryptBlockHeader {
    #[serde(with = "hex_array_80")]
    pub block_header: [u8; 80],
    #[serde(with = "hex_array_32")]
    pub scrypt_hash: [u8; 32],
}


fn run_get_block_hashes(
    start_block: u32,
    total: usize,
    client: DogeLinkElectrsClient,
    file_path: &str,
) -> anyhow::Result<()> {
    let end_block = start_block + total as u32;

    let mut cache = BlockHeaderFetcher::new(client);
    cache.load_block_headers_bin(file_path)?;

    let binding = (start_block..end_block).collect::<Vec<u32>>();
    let chunks = binding.chunks(10);

    let mut headers = Vec::with_capacity(total);
    for c in chunks {
        let blocks = cache.get_block_headers(c)?;
        for block in blocks {
            let auxpow= block.aux_pow.unwrap();
            let header = DogeScryptBlockHeader {
                block_header: auxpow.parent_block.to_bytes_fixed(),
                scrypt_hash: auxpow.parent_block.get_pow_hash(),
            };
            headers.push(header);
        }
    }

    println!("headers:\n{}", serde_json::to_string_pretty(&headers)?);
    Ok(())
}

fn main() {
    run_get_block_hashes(
        5610384,
        500,
        DogeLinkElectrsClient::new(
            "https://doge-electrs-demo.qed.me".to_string(),
            DogeNetworkType::MainNet,
        ),
        "test_data/mainnet_headers_5620352-5621352.bin",
    )
    .unwrap();
}
