use doge_light_client::common_types::QHash256;


pub struct PsyBridgeStateCommitment {
    pub block_hash: QHash256,
    pub block_merkle_tree_root: QHash256,
    pub auto_processed_dtxo_tree_root: QHash256,
}
pub struct PsyBridgeHeader {
    pub tip_state: PsyBridgeStateCommitment,
    pub finalized_state: PsyBridgeStateCommitment,
    pub bridge_state_hash: QHash256,

    pub tip_block_height: u32,
    pub finalized_block_height: u32,
    pub last_rollback_at_secs: u32,
    pub paused_until_secs: u32,

    pub auto_collected_fees_balance_sats: u64,
}