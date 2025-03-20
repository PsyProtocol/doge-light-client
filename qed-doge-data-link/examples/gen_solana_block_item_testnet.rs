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

use doge_light_client::network_params::DogeNetworkType;
use qed_doge_data_link::{
    block_header_cache::BlockHeaderFetcher, electrs_link::DogeLinkElectrsClient,
};

use qed_doge_data_link::hex_helpers::{hex_array_32, hex_array_80};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;


#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct SolanaDogeIBCBlockItem {
    #[serde(with = "hex_array_80")]
    pub aux_pow_block_header: [u8; 80],

    #[serde(with = "hex_array_32")]
    pub aux_pow_block_header_scrypt_hash: [u8; 32],

    #[serde_as(as = "serde_with::hex::Hex")]
    pub raw_block_header: Vec<u8>,
}

fn main() {

    let mut fetcher = BlockHeaderFetcher::new(DogeLinkElectrsClient::new(
        "https://doge-electrs-testnet-demo.qed.me".to_string(),
        DogeNetworkType::TestNet,
    ));

    fetcher
        .load_block_headers_bin("test_data/testnet_block_headers_7654400-7654500.bin")
        .unwrap();

    let from_block = 7667430;
    let to_block = from_block + 4;

    let headers = fetcher
        .get_block_headers(&(from_block..to_block).collect::<Vec<_>>())
        .unwrap();

    let mut results = Vec::with_capacity(headers.len());
    for h in headers.iter() {
        match &h.aux_pow {
            Some(aux_pow) => {
                let aux_pow_block_header = aux_pow.parent_block.to_bytes_fixed();
                println!("hash: {}", hex::encode(&h.header.get_hash()));
                let aux_pow_block_header_scrypt_hash = aux_pow.parent_block.get_pow_hash();
                let raw_block_header = borsh::to_vec(&h).unwrap();
                results.push(SolanaDogeIBCBlockItem {
                    aux_pow_block_header,
                    aux_pow_block_header_scrypt_hash,
                    raw_block_header,
                });
            }
            None => {
                results.push(SolanaDogeIBCBlockItem {
                    aux_pow_block_header: h.header.to_bytes_fixed(),
                    aux_pow_block_header_scrypt_hash: h.header.get_pow_hash(),
                    raw_block_header: borsh::to_vec(&h).unwrap(),
                });
            }
        }
    }

    println!("results:\n{}", serde_json::to_string_pretty(&results).unwrap());
}
