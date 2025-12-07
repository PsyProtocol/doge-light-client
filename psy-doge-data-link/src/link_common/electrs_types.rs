use serde::{Deserialize, Serialize};

use crate::link_common::wrapped_hash_256::WrappedHash256;

// ============================================================================
// Block Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsBlock {
    pub id: WrappedHash256,
    pub height: u32,
    pub version: u32,
    pub timestamp: u32,
    pub tx_count: u32,
    pub size: u32,
    pub weight: u64,
    pub merkle_root: WrappedHash256,
    pub previousblockhash: Option<WrappedHash256>,
    pub mediantime: u32,
    pub nonce: u32,
    pub bits: u32,
    pub difficulty: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsBlockStatus {
    pub in_best_chain: bool,
    pub height: Option<u32>,
    pub next_best: Option<WrappedHash256>,
}

// ============================================================================
// Transaction Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsTransaction {
    pub txid: WrappedHash256,
    pub version: u32,
    pub locktime: u32,
    pub vin: Vec<ElectrsTxIn>,
    pub vout: Vec<ElectrsTxOut>,
    pub size: u32,
    pub weight: u64,
    pub fee: u64,
    pub status: ElectrsTxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsTxIn {
    pub txid: WrappedHash256,
    pub vout: u32,
    pub prevout: Option<ElectrsTxOut>,
    pub scriptsig: String,
    pub scriptsig_asm: String,
    #[serde(default)]
    pub witness: Option<Vec<String>>,
    pub is_coinbase: bool,
    pub sequence: u32,
    pub inner_redeemscript_asm: Option<String>,
    pub inner_witnessscript_asm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsTxOut {
    pub scriptpubkey: String,
    pub scriptpubkey_asm: String,
    pub scriptpubkey_type: String,
    pub scriptpubkey_address: Option<String>,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsTxStatus {
    pub confirmed: bool,
    pub block_height: Option<u32>,
    pub block_hash: Option<WrappedHash256>,
    pub block_time: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsSpendingStatus {
    pub spent: bool,
    pub txid: Option<WrappedHash256>,
    pub vin: Option<u32>,
    pub status: Option<ElectrsTxStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsMerkleProof {
    pub block_height: u32,
    pub merkle: Vec<WrappedHash256>,
    pub pos: u32,
}

// Package API (Bitcoin Core 28.0+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageResult {
    pub txid: WrappedHash256,
    pub other_info: Option<serde_json::Value>, // Placeholder for variable response
}

// ============================================================================
// Address / Script / Stats Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsUtxo {
    pub txid: WrappedHash256,
    pub vout: u32,
    pub value: u64,
    pub status: ElectrsTxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsAddressStats {
    pub funded_txo_count: u32,
    pub funded_txo_sum: u64,
    pub spent_txo_count: u32,
    pub spent_txo_sum: u64,
    pub tx_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsStatsResponse {
    #[serde(alias = "address", alias = "scripthash")]
    pub target: String,
    pub chain_stats: ElectrsAddressStats,
    pub mempool_stats: ElectrsAddressStats,
}

// ============================================================================
// Mempool Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsMempoolStats {
    pub count: u32,
    pub vsize: u32,
    pub total_fee: u64,
    pub fee_histogram: Vec<(u32, u32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectrsMempoolRecent {
    pub txid: WrappedHash256,
    pub fee: u64,
    pub vsize: u32,
    pub value: u64,
}