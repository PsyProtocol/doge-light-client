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


use qed_doge_data_link::{electrs_link::DogeLinkElectrsClient, traits::QDogeBlockHeaderFetcher, wrapped_hash_256::WrappedHash256};
use doge_light_client::{core_data::QHash256, hash::sha256::QBTCHash256Hasher, network_params::DogeNetworkType};
fn run_merkle_test(
    txids: &[QHash256],
    client: DogeLinkElectrsClient,
) -> anyhow::Result<()> {

    for txid in txids.iter() {
        println!("txid: {}", WrappedHash256(*txid).to_hex_string());
        let e_proof = client.get_electrum_merkle_proof_from_txid(*txid)?;
        let mut q_proof = e_proof.to_merkle_proof_core_with_txid(*txid);
        let blk = client.get_qdoge_block_header(e_proof.block_height)?;
        println!("computed_root: {}, actual_root: {}", WrappedHash256(q_proof.root).to_hex_string(), WrappedHash256(blk.header.merkle_root).to_hex_string());

        assert_eq!(blk.header.merkle_root, q_proof.root, "merkle root mismatch, expected: {}, got {}", WrappedHash256(blk.header.merkle_root).to_hex_string(), WrappedHash256(q_proof.root).to_hex_string());
        assert!(q_proof.verify_btc_block_tx_tree::<QBTCHash256Hasher>(),"merkle proof verification failed");
        for _ in 0..256 {
            q_proof.index += 1;
            assert!(!q_proof.verify_btc_block_tx_tree::<QBTCHash256Hasher>(),"merkle proof verification should have failed if we changed the index");
        }
    }
    Ok(())
}

fn main() {




    



    run_merkle_test(
       &[
        // block of 2
        hex_literal::hex!("4866a74e2ca5bc2cc9f6178e555e4ba9456c188a76e4c4cd6b4cef8403c50106"),
        hex_literal::hex!("7760d800117801e43c0a6c822431819b4dbade8465ec8e1ec4b0972c2107c1a6"),
        
        
        //block of 3
        hex_literal::hex!("c4691e347c953b1adb23109fc63e3468fcb23d907e5dd5e5f8c2f87a509e2021"),
        hex_literal::hex!("48aa075ba24ea7f2b96a116f0a63e4eceea176584a5d2a12df4e81a021948288"),
        hex_literal::hex!("41c40f723771999c6b28f95ebe1f2051a4fae5b8eba622d6d93ccf30a0897521"),


        hex_literal::hex!("796136adb594915b0ccd8617d533190a25810b283d4456e44e18f00a1691b6f4"),
       ],
        DogeLinkElectrsClient::new(
            "https://doge-electrs-testnet-demo.qed.me".to_string(),
            DogeNetworkType::TestNet,
        ),
    )
    .unwrap();
}
