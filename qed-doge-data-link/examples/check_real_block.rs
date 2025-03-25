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

use doge_light_client::{
    core_data::QHash256, hash::sha256::QSha256Hasher, network_params::DogeNetworkType,
};
use qed_doge_bridge_helper::{
    bridge_tx_proof_v1::{
        TransactionInBlockProofV1, UserClaimStateProofV1, get_user_claimed_combined_index,
        with_set_bit_in_bit_vector,
    },
    tx_template::get_bridge_deposit_address_v1,
};
use qed_doge_data_link::{
    block_header_cache::BlockHeaderFetcher,
    bridge_state_helpers::gen_bridge_initial_state,
    electrs_link::DogeLinkElectrsClient,
    simple_merkle_tree::SimpleMerkleTree,
};

fn run_check_real_block(
    tx_block_number: u32,
    txid: [u8; 32],
    solana_address: [u8; 32],
) -> anyhow::Result<()> {
    const QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS: usize = 4;
    const QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE: usize = 32;
    const QDOGE_BRIDGE_BLOCK_TREE_HEIGHT: usize = 32;
    const BRIDGE_PUBLIC_KEY_HASH: [u8; 20] =
        hex_literal::hex!("9e53cfc8118221f1d31833c2be034155fd3488d4");

    let mut fetcher = BlockHeaderFetcher::new(DogeLinkElectrsClient::new(
        "https://doge-electrs-testnet-demo.qed.me".to_string(),
        DogeNetworkType::TestNet,
    ));

    let bridge_deposit_address =
        get_bridge_deposit_address_v1(&solana_address, &BRIDGE_PUBLIC_KEY_HASH);

    let new_tip = tx_block_number + QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS as u32;
    let ibc = gen_bridge_initial_state::<
        _,
        QDOGE_BRIDGE_BLOCK_HASH_CACHE_SIZE,
        QDOGE_BRIDGE_REQUIRED_CONFIRMATIONS,
        QDOGE_BRIDGE_BLOCK_TREE_HEIGHT,
    >(&mut fetcher, new_tip)?;

    let block_merkle_proof = fetcher.client.get_q_merkle_proof_from_txid(txid)?;

    let tx = fetcher.client.get_q_tx_from_txid(txid)?;

    let tx_output_index = *(tx
        .get_vouts_for_address(&bridge_deposit_address)
        .last()
        .unwrap());
    let tx_index_in_block = block_merkle_proof.index as u32;
    let (combo_index, bit_vector_index) =
        get_user_claimed_combined_index(tx_block_number, tx_index_in_block, tx_output_index);

    let mut user_tree = SimpleMerkleTree::<QSha256Hasher, QHash256>::new(64);

    let user_claim_bit_vector = user_tree.get_leaf_value(combo_index);

    let update_user_claim_dmp = user_tree.set_leaf(
        combo_index,
        with_set_bit_in_bit_vector(&user_claim_bit_vector, bit_vector_index),
    );
    let known_user_claim_merkle_hash = update_user_claim_dmp.old_root;

    let siblings: [[u8; 32]; 64] = update_user_claim_dmp.siblings.clone().try_into().unwrap();

    let tx_in_block_proof = TransactionInBlockProofV1::new(block_merkle_proof.siblings, tx);

    let user_claim_state_proof =
        UserClaimStateProofV1::new(tx_in_block_proof, user_claim_bit_vector, siblings);

    let ser_result = user_claim_state_proof.to_bytes();

    let (new_root, amount) = UserClaimStateProofV1::verify_tx_out_in_block_is_deposit_v1_with_ibc(
        &solana_address,
        &BRIDGE_PUBLIC_KEY_HASH,
        tx_block_number,
        tx_index_in_block,
        tx_output_index,
        &ibc,
        &known_user_claim_merkle_hash,
        &ser_result,
    )?;

    println!("verified claim for amount: {}", amount);

    println!("new_user_claim_root: {}", hex::encode(&new_root));

    Ok(())
}
fn main() {
    run_check_real_block(
        7703182,
        hex_literal::hex!("9284df9e4d6223f74711eab5b3546cd6cfa99c0f60045aabce860d45c71f8eb8"),
        hex_literal::hex!("e83c24b97aeadd8de838b7c040347ac9e821a103c38b2999a7989f7a6181e0d8"),
    ).unwrap();
}
