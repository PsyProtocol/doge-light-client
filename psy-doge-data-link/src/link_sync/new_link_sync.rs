use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

use doge_light_client::{common_types::QHash256, doge::transaction::BTCTransaction};
use ureq::{Agent, Body, http::Response};

// Import the types we defined above
use crate::link_common::electrs_types::{
    ElectrsBlock, ElectrsBlockStatus, ElectrsMempoolRecent, ElectrsMempoolStats,
    ElectrsMerkleProof, ElectrsSpendingStatus, ElectrsStatsResponse, ElectrsTransaction,
    ElectrsTxStatus, ElectrsUtxo,
};


#[derive(Debug, Clone)]
pub struct DogeElectrsClientSync {
    base_url: String,
    client: Agent,
}

impl DogeElectrsClientSync {
    /// Create a new Electrs client
    ///
    /// `url`: The base URL of the electrs instance (e.g. "https://blockstream.info/api")
    pub fn new(url: String) -> Self {
        let client = Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build()
            .into();

        // Ensure no trailing slash for easier path concatenation
        let base_url = url.trim_end_matches('/').to_string();

        Self { base_url, client }
    }

    // ========================================================================
    // Helpers
    // ========================================================================

    /// Converts a QHash256 ([u8; 32] Little Endian) to a Hex String (Big Endian)
    /// required for URL parameters in the Electrs API.
    fn hash_to_hex_rpc(hash: &QHash256) -> String {
        let mut reversed = *hash;
        reversed.reverse();
        hex::encode(reversed)
    }

    /// Helper to determine the URL path segment for address vs scripthash
    fn get_target_path(address_or_hash: &str) -> String {
        // Simple heuristic: if it's 64 hex chars, assume script hash, otherwise address
        if address_or_hash.len() == 64 && hex::decode(address_or_hash).is_ok() {
            format!("scripthash/{}", address_or_hash)
        } else {
            format!("address/{}", address_or_hash)
        }
    }

    /// Internal helper to unify error handling for ureq requests
    fn call_get(&self, endpoint: &str) -> Result<Response<Body>> {
        let url = format!("{}/{}", self.base_url, endpoint);
        match self.client.get(&url).call() {
            Ok(resp) => Ok(resp),
            Err(ureq::Error::StatusCode(code)) => {
                anyhow::bail!("HTTP Request failed: {}", code);
            }
            Err(e) => {
                anyhow::bail!("HTTP Transport failed for {}: {}", url, e);
            }
        }
    }

    fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let resp = self.call_get(endpoint)?;
        resp.into_body().read_json::<T>()
            .context(format!("Failed to parse JSON from {}/{}", self.base_url, endpoint))
    }

    fn get_text(&self, endpoint: &str) -> Result<String> {
        let resp = self.call_get(endpoint)?;
        resp.into_body().read_to_string().context("Failed to get text response")
    }

    fn get_bytes(&self, endpoint: &str) -> Result<Vec<u8>> {
        let resp = self.call_get(endpoint)?;
            
        resp.into_body()
            .read_to_vec()
            .context("Failed to get byte response")
    }

    // ========================================================================
    // Transaction Endpoints
    // ========================================================================

    /// GET /tx/:txid
    pub fn get_transaction_info(&self, txid: &QHash256) -> Result<ElectrsTransaction> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}", txid_str))
    }

    /// GET /tx/:txid/status
    pub fn get_transaction_status(&self, txid: &QHash256) -> Result<ElectrsTxStatus> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/status", txid_str))
    }

    /// GET /tx/:txid/hex
    pub fn get_transaction_hex(&self, txid: &QHash256) -> Result<String> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_text(&format!("tx/{}/hex", txid_str))
    }

    /// GET /tx/:txid/raw
    pub fn get_transaction_raw(&self, txid: &QHash256) -> Result<Vec<u8>> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_bytes(&format!("tx/{}/raw", txid_str))
    }

    /// Helper that calls /raw and parses into BTCTransaction
    pub fn get_transaction(&self, txid: &QHash256) -> Result<BTCTransaction> {
        let bytes = self.get_transaction_raw(txid)?;
        BTCTransaction::from_bytes(&bytes).context("Failed to deserialize BTCTransaction")
    }

    /// GET /tx/:txid/merkleblock-proof
    /// Returns the raw hex string of the BIP37 merkleblock
    pub fn get_transaction_merkleblock_proof_hex(&self, txid: &QHash256) -> Result<String> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_text(&format!("tx/{}/merkleblock-proof", txid_str))
    }

    /// GET /tx/:txid/merkle-proof
    /// Returns Electrum's blockchain.transaction.get_merkle format
    pub fn get_transaction_merkle_proof(
        &self,
        txid: &QHash256,
    ) -> Result<ElectrsMerkleProof> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/merkle-proof", txid_str))
    }

    /// GET /tx/:txid/outspend/:vout
    pub fn get_transaction_outspend(
        &self,
        txid: &QHash256,
        vout: u32,
    ) -> Result<ElectrsSpendingStatus> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/outspend/{}", txid_str, vout))
    }

    /// GET /tx/:txid/outspends
    pub fn get_transaction_outspends(
        &self,
        txid: &QHash256,
    ) -> Result<Vec<ElectrsSpendingStatus>> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/outspends", txid_str))
    }

    /// POST /tx
    /// Broadcast a raw transaction
    pub fn broadcast_raw(&self, tx_hex: &str) -> Result<String> {
        let url = format!("{}/tx", self.base_url);
        
        let resp = match self.client.post(&url).send(tx_hex.as_bytes()) {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(code)) => {
                anyhow::bail!("Broadcast failed ({})", code);
            }
            Err(e) => anyhow::bail!("Broadcast transport failed: {}", e),
        };

        // Returns txid on success
        resp.into_body().read_to_string()
            .context("Failed to get broadcast response")
    }

    /// Helper to broadcast a BTCTransaction object
    pub fn broadcast_transaction(&self, tx: &BTCTransaction) -> Result<String> {
        let hex_str = hex::encode(tx.to_bytes());
        self.broadcast_raw(&hex_str)
    }

    /// POST /txs/package
    /// Requires Bitcoin Core 28.0+
    pub fn broadcast_package(&self, tx_hexs: Vec<String>) -> Result<serde_json::Value> {
        let url = format!("{}/txs/package", self.base_url);
        
        let resp = match self.client.post(&url).send_json(&tx_hexs) {
            Ok(r) => r,
            Err(ureq::Error::StatusCode(code)) => {
                anyhow::bail!("Package broadcast failed ({})", code);
            }
            Err(e) => anyhow::bail!("Package broadcast transport failed: {}", e),
        };

        resp.into_body().read_json::<serde_json::Value>()
            .context("Failed to parse package response")
    }

    // ========================================================================
    // Address / Scripthash Endpoints
    // ========================================================================

    /// GET /address/:address OR /scripthash/:hash
    pub fn get_stats(&self, address_or_hash: &str) -> Result<ElectrsStatsResponse> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&path)
    }

    /// GET /address/:address/txs
    /// Returns 50 mempool txs + 25 confirmed txs
    pub fn get_address_txs(&self, address_or_hash: &str) -> Result<Vec<ElectrsTransaction>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/txs", path))
    }

    /// GET /address/:address/txs/chain[/:last_seen_txid]
    /// Returns 25 confirmed txs per page
    pub fn get_address_txs_chain(
        &self,
        address_or_hash: &str,
        last_seen_txid: Option<&str>,
    ) -> Result<Vec<ElectrsTransaction>> {
        let path = Self::get_target_path(address_or_hash);
        let endpoint = if let Some(txid) = last_seen_txid {
            format!("{}/txs/chain/{}", path, txid)
        } else {
            format!("{}/txs/chain", path)
        };
        self.get_json(&endpoint)
    }

    /// GET /address/:address/txs/mempool
    /// Returns up to 50 unconfirmed txs
    pub fn get_address_txs_mempool(
        &self,
        address_or_hash: &str,
    ) -> Result<Vec<ElectrsTransaction>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/txs/mempool", path))
    }

    /// GET /address/:address/utxo
    pub fn get_utxos(&self, address_or_hash: &str) -> Result<Vec<ElectrsUtxo>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/utxo", path))
    }

    /// GET /address-prefix/:prefix
    pub fn search_address_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        self.get_json(&format!("address-prefix/{}", prefix))
    }

    // ========================================================================
    // Block Endpoints
    // ========================================================================

    /// GET /block/:hash
    pub fn get_block_info(&self, hash: &QHash256) -> Result<ElectrsBlock> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_json(&format!("block/{}", hash_str))
    }

    /// GET /block/:hash/header
    /// Returns hex-encoded block header
    pub fn get_block_header_hex(&self, hash: &QHash256) -> Result<String> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_text(&format!("block/{}/header", hash_str))
    }

    /// GET /block/:hash/status
    pub fn get_block_status(&self, hash: &QHash256) -> Result<ElectrsBlockStatus> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_json(&format!("block/{}/status", hash_str))
    }

    /// GET /block/:hash/txs[/:start_index]
    pub fn get_block_txs(
        &self,
        hash: &QHash256,
        start_index: Option<u32>,
    ) -> Result<Vec<ElectrsTransaction>> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        let endpoint = if let Some(idx) = start_index {
            format!("block/{}/txs/{}", hash_str, idx)
        } else {
            format!("block/{}/txs", hash_str)
        };
        self.get_json(&endpoint)
    }

    /// GET /block/:hash/txids
    /// Returns all txids in block
    pub fn get_block_txids(&self, hash: &QHash256) -> Result<Vec<QHash256>> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        let txids_hex: Vec<String> = self.get_json(&format!("block/{}/txids", hash_str))?;

        // Convert hex strings back to QHash256 (reversing back to LE)
        let mut result = Vec::with_capacity(txids_hex.len());
        for hex_str in txids_hex {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(&hex_str, &mut bytes).context("Invalid txid hex in response")?;
            bytes.reverse(); // BE -> LE
            result.push(bytes);
        }
        Ok(result)
    }

    /// GET /block/:hash/txid/:index
    /// Returns txid at specific index
    pub fn get_block_txid_at_index(&self, hash: &QHash256, index: u32) -> Result<QHash256> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        let hex_str = self
            .get_text(&format!("block/{}/txid/{}", hash_str, index))?;

        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid txid hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    /// GET /block/:hash/raw
    pub fn get_block_raw(&self, hash: &QHash256) -> Result<Vec<u8>> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_bytes(&format!("block/{}/raw", hash_str))
    }

    /// GET /block-height/:height
    /// Returns the hash of the block at :height
    pub fn get_block_hash(&self, height: u32) -> Result<QHash256> {
        let hex_str = self.get_text(&format!("block-height/{}", height))?;
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid block hash hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    /// GET /blocks[/:start_height]
    /// Returns 10 newest blocks
    pub fn get_blocks(&self, start_height: Option<u32>) -> Result<Vec<ElectrsBlock>> {
        let endpoint = if let Some(h) = start_height {
            format!("blocks/{}", h)
        } else {
            "blocks".to_string()
        };
        self.get_json(&endpoint)
    }

    /// GET /blocks/tip/height
    pub fn get_block_height(&self) -> Result<u32> {
        let text = self.get_text("blocks/tip/height")?;
        text.parse::<u32>().context("Failed to parse block height")
    }

    /// GET /blocks/tip/hash
    pub fn get_tip_hash(&self) -> Result<QHash256> {
        let hex_str = self.get_text("blocks/tip/hash")?;
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid tip hash hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    // ========================================================================
    // General / Mempool Endpoints
    // ========================================================================

    /// GET /mempool
    pub fn get_mempool_stats(&self) -> Result<ElectrsMempoolStats> {
        self.get_json("mempool")
    }

    /// GET /mempool/txids
    pub fn get_mempool_txids(&self) -> Result<Vec<QHash256>> {
        let txids_hex: Vec<String> = self.get_json("mempool/txids")?;
        // Convert hex strings back to QHash256 (reversing back to LE)
        let mut result = Vec::with_capacity(txids_hex.len());
        for hex_str in txids_hex {
            let mut bytes = [0u8; 32];
            hex::decode_to_slice(&hex_str, &mut bytes).context("Invalid txid hex in response")?;
            bytes.reverse();
            result.push(bytes);
        }
        Ok(result)
    }

    /// GET /mempool/recent
    pub fn get_mempool_recent(&self) -> Result<Vec<ElectrsMempoolRecent>> {
        self.get_json("mempool/recent")
    }

    /// GET /fee-estimates
    pub fn get_fee_estimates(&self) -> Result<HashMap<String, f64>> {
        self.get_json("fee-estimates")
    }
}