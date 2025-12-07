use borsh::BorshDeserialize;
use doge_light_client::{
    block_state::PsyBridgeHeader, common_types::{QHash160, QHash256}, constants::DogeNetworkConfig, hash::sha256_impl::{hash_impl_btc_hash256_two_to_one_bytes, hash_impl_sha256_bytes}
};
use speedy::Readable;
use zerocopy::IntoBytes;

use crate::
    data::core::{PsyDogeBridgeIncomingBlockWitness, PsyDogeBridgeState}
;

pub fn prover_guest_run_with_bytes(
    data: &[u8],
) -> anyhow::Result<(PsyDogeBridgeIncomingBlockWitness, PsyDogeBridgeState)> {
    if data.len() < 4 {
        return Err(anyhow::anyhow!(
            "data length too small to contain state length"
        ));
    }

    let state_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if data.len() < 4 + state_len {
        return Err(anyhow::anyhow!(
            "data length too small to contain state data"
        ));
    }
    let witness = PsyDogeBridgeIncomingBlockWitness::read_from_buffer(&data[4 + state_len..])?;
    let state = PsyDogeBridgeState::try_from_slice(&data[4..4 + state_len])?;
    // Return references to avoid cloning

    Ok((witness, state))
}
pub fn prover_guest_verify_block_transition_bytes<NC: DogeNetworkConfig>(
    bridge_public_key_hash: QHash160,
    required_confirmations: u32,
    flat_fee_per_deposit_sats: u64,
    deposit_fee_rate_numerator: u64,
    deposit_fee_rate_denominator: u64,
    input_data: &[u8],
) -> anyhow::Result<QHash256> {
    let (witness, mut state) = prover_guest_run_with_bytes(input_data)?;
    prover_guest_verify_block_transition::<NC>(
        bridge_public_key_hash,
        required_confirmations,
        witness,
        &mut state,
        flat_fee_per_deposit_sats,
        deposit_fee_rate_numerator,
        deposit_fee_rate_denominator,
    )
}
pub fn prover_guest_verify_block_transition<NC: DogeNetworkConfig>(
    bridge_public_key_hash: QHash160,
    required_confirmations: u32,
    witness: PsyDogeBridgeIncomingBlockWitness,
    state: &mut PsyDogeBridgeState,
    flat_fee_per_deposit_sats: u64,
    deposit_fee_rate_numerator: u64,
    deposit_fee_rate_denominator: u64,
) -> anyhow::Result<QHash256> {
    let block_number = state.get_tip_block_number() + 1;
    let latest = state
        .block_data_tracker
        .get_record_ref(state.get_tip_block_number())?;

    //let previous_record = state.block_data_tracker.get_record(state.get_tip_block_number())?;
    let old_state_hash = hash_impl_sha256_bytes(state.as_bytes());
    let previous_header = PsyBridgeHeader {
        tip_state: state.block_data_tracker.get_tip_state_commitment(),
        finalized_state: state.block_data_tracker.get_finalized_state_commitment(required_confirmations)?,
        bridge_state_hash: old_state_hash,
        last_rollback_at_secs: witness.previous_header_last_rollback_at_secs,
        paused_until_secs: witness.previous_header_paused_until_secs,
        total_finalized_fees_collected_chain_history: state.block_data_tracker.get_record(state.get_finalized_block_number(required_confirmations))?.total_fees_collected_chain_history.into(),
    };

    if witness.block_header.header.timestamp < previous_header.paused_until_secs {
        return Err(anyhow::anyhow!(
            "Block timestamp {} is before previous paused_until_secs {}",
            witness.block_header.header.timestamp,
            previous_header.paused_until_secs
        ));
    }
    let previous_header_hash = previous_header.get_hash_canonical();

    let expected_start_auto_claimed_deposits_tree_root = latest.auto_claimed_deposits_tree_root;
    let expected_start_auto_claimed_deposits_index = latest.auto_claimed_deposits_next_index;
    let expected_start_claimed_txo_tree_root = latest.auto_claimed_txo_tree_root;

    let result = witness.claim_witness.verify_and_get_result(
        block_number,
        witness.block_header.header.merkle_root,
        bridge_public_key_hash,
        flat_fee_per_deposit_sats,
        deposit_fee_rate_numerator,
        deposit_fee_rate_denominator,
    )?;

    if result.start_auto_claimed_deposits_index != expected_start_auto_claimed_deposits_index.into()
    {
        return Err(anyhow::anyhow!(
            "start_auto_claimed_deposits_index mismatch: expected {}, got {}",
            expected_start_auto_claimed_deposits_index,
            result.start_auto_claimed_deposits_index
        ));
    }
    if result.old_auto_claimed_deposits_tree_root != expected_start_auto_claimed_deposits_tree_root
    {
        return Err(anyhow::anyhow!(
            "start_auto_claimed_deposits_tree_root mismatch: expected {:?}, got {:?}",
            expected_start_auto_claimed_deposits_tree_root,
            result.old_auto_claimed_deposits_tree_root
        ));
    }
    if result.old_claimed_txo_tree_root != expected_start_claimed_txo_tree_root {
        return Err(anyhow::anyhow!(
            "start_claimed_txo_tree_root mismatch: expected {:?}, got {:?}",
            expected_start_claimed_txo_tree_root,
            result.old_claimed_txo_tree_root
        ));
    }

    state.append_block::<NC>(
        block_number,
        &witness.block_header,
        result.new_claimed_txo_tree_root,
        result.new_auto_claimed_deposits_tree_root,
        result.end_auto_claimed_deposits_index,
        result.fees_collected,
        None,
    )?;
    let new_state_hash = hash_impl_sha256_bytes(state.as_bytes());

    //let new_tip_record = state.block_data_tracker.get_record(state.get_tip_block_number())?;
    let new_header = PsyBridgeHeader {
        tip_state: state.block_data_tracker.get_tip_state_commitment(),
        finalized_state: state.block_data_tracker.get_finalized_state_commitment(required_confirmations)?,
        bridge_state_hash: new_state_hash,
        last_rollback_at_secs: previous_header.last_rollback_at_secs,
        paused_until_secs: previous_header.paused_until_secs,
        total_finalized_fees_collected_chain_history: state.block_data_tracker.get_record(state.get_finalized_block_number(required_confirmations))?.total_fees_collected_chain_history.into(),
    };

    let new_header_hash = new_header.get_hash_canonical();


    let final_hash = hash_impl_btc_hash256_two_to_one_bytes(
        &previous_header_hash,
        &new_header_hash,
    );
    Ok(final_hash)
}
