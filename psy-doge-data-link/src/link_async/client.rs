use std::collections::HashMap;
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::de::DeserializeOwned;

use doge_light_client::{common_types::QHash256, doge::transaction::BTCTransaction};

// Import the types we defined above
use crate::link_common::electrs_types::{
    ElectrsBlock, ElectrsBlockStatus, ElectrsMempoolRecent, ElectrsMempoolStats,
    ElectrsMerkleProof, ElectrsSpendingStatus, ElectrsStatsResponse, ElectrsTransaction,
    ElectrsTxStatus, ElectrsUtxo,
};

#[derive(Debug, Clone)]
pub struct DogeElectrsClient {
    base_url: String,
    client: Client,
}

impl DogeElectrsClient {
    /// Create a new Electrs client
    ///
    /// `url`: The base URL of the electrs instance (e.g. "https://blockstream.info/api")
    pub fn new(url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

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

    async fn get_json<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("HTTP Request failed: {} - {}", status, text);
        }

        resp.json::<T>()
            .await
            .context(format!("Failed to parse JSON from {}", url))
    }

    async fn get_text(&self, endpoint: &str) -> Result<String> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("HTTP Request failed: {}", resp.status());
        }

        resp.text().await.context("Failed to get text response")
    }

    async fn get_bytes(&self, endpoint: &str) -> Result<Vec<u8>> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            anyhow::bail!("HTTP Request failed: {}", resp.status());
        }

        let b = resp.bytes().await.context("Failed to get byte response")?;
        Ok(b.to_vec())
    }

    // ========================================================================
    // Transaction Endpoints
    // ========================================================================

    /// GET /tx/:txid
    pub async fn get_transaction_info(&self, txid: &QHash256) -> Result<ElectrsTransaction> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}", txid_str)).await
    }

    /// GET /tx/:txid/status
    pub async fn get_transaction_status(&self, txid: &QHash256) -> Result<ElectrsTxStatus> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/status", txid_str)).await
    }

    /// GET /tx/:txid/hex
    pub async fn get_transaction_hex(&self, txid: &QHash256) -> Result<String> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_text(&format!("tx/{}/hex", txid_str)).await
    }

    /// GET /tx/:txid/raw
    pub async fn get_transaction_raw(&self, txid: &QHash256) -> Result<Vec<u8>> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_bytes(&format!("tx/{}/raw", txid_str)).await
    }

    /// Helper that calls /raw and parses into BTCTransaction
    pub async fn get_transaction(&self, txid: &QHash256) -> Result<BTCTransaction> {
        let bytes = self.get_transaction_raw(txid).await?;
        BTCTransaction::from_bytes(&bytes).context("Failed to deserialize BTCTransaction")
    }

    /// GET /tx/:txid/merkleblock-proof
    /// Returns the raw hex string of the BIP37 merkleblock
    pub async fn get_transaction_merkleblock_proof_hex(&self, txid: &QHash256) -> Result<String> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_text(&format!("tx/{}/merkleblock-proof", txid_str))
            .await
    }

    /// GET /tx/:txid/merkle-proof
    /// Returns Electrum's blockchain.transaction.get_merkle format
    pub async fn get_transaction_merkle_proof(
        &self,
        txid: &QHash256,
    ) -> Result<ElectrsMerkleProof> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/merkle-proof", txid_str))
            .await
    }

    /// GET /tx/:txid/outspend/:vout
    pub async fn get_transaction_outspend(
        &self,
        txid: &QHash256,
        vout: u32,
    ) -> Result<ElectrsSpendingStatus> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/outspend/{}", txid_str, vout))
            .await
    }

    /// GET /tx/:txid/outspends
    pub async fn get_transaction_outspends(
        &self,
        txid: &QHash256,
    ) -> Result<Vec<ElectrsSpendingStatus>> {
        let txid_str = Self::hash_to_hex_rpc(txid);
        self.get_json(&format!("tx/{}/outspends", txid_str)).await
    }

    /// POST /tx
    /// Broadcast a raw transaction
    pub async fn broadcast_raw(&self, tx_hex: &str) -> Result<String> {
        let url = format!("{}/tx", self.base_url);
        let resp = self
            .client
            .post(&url)
            .body(tx_hex.to_string())
            .send()
            .await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Broadcast failed: {}", err);
        }
        // Returns txid on success
        resp.text()
            .await
            .context("Failed to get broadcast response")
    }

    /// Helper to broadcast a BTCTransaction object
    pub async fn broadcast_transaction(&self, tx: &BTCTransaction) -> Result<String> {
        let hex_str = hex::encode(tx.to_bytes());
        self.broadcast_raw(&hex_str).await
    }

    /// POST /txs/package
    /// Requires Bitcoin Core 28.0+
    pub async fn broadcast_package(&self, tx_hexs: Vec<String>) -> Result<serde_json::Value> {
        let url = format!("{}/txs/package", self.base_url);
        let resp = self.client.post(&url).json(&tx_hexs).send().await?;

        if !resp.status().is_success() {
            let err = resp.text().await.unwrap_or_default();
            anyhow::bail!("Package broadcast failed: {}", err);
        }
        resp.json()
            .await
            .context("Failed to parse package response")
    }

    // ========================================================================
    // Address / Scripthash Endpoints
    // ========================================================================

    /// GET /address/:address OR /scripthash/:hash
    pub async fn get_stats(&self, address_or_hash: &str) -> Result<ElectrsStatsResponse> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&path).await
    }

    /// GET /address/:address/txs
    /// Returns 50 mempool txs + 25 confirmed txs
    pub async fn get_address_txs(&self, address_or_hash: &str) -> Result<Vec<ElectrsTransaction>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/txs", path)).await
    }

    /// GET /address/:address/txs/chain[/:last_seen_txid]
    /// Returns 25 confirmed txs per page
    pub async fn get_address_txs_chain(
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
        self.get_json(&endpoint).await
    }

    /// GET /address/:address/txs/mempool
    /// Returns up to 50 unconfirmed txs
    pub async fn get_address_txs_mempool(
        &self,
        address_or_hash: &str,
    ) -> Result<Vec<ElectrsTransaction>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/txs/mempool", path)).await
    }

    /// GET /address/:address/utxo
    pub async fn get_utxos(&self, address_or_hash: &str) -> Result<Vec<ElectrsUtxo>> {
        let path = Self::get_target_path(address_or_hash);
        self.get_json(&format!("{}/utxo", path)).await
    }

    /// GET /address-prefix/:prefix
    pub async fn search_address_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        self.get_json(&format!("address-prefix/{}", prefix)).await
    }

    // ========================================================================
    // Block Endpoints
    // ========================================================================

    /// GET /block/:hash
    pub async fn get_block_info(&self, hash: &QHash256) -> Result<ElectrsBlock> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_json(&format!("block/{}", hash_str)).await
    }

    /// GET /block/:hash/header
    /// Returns hex-encoded block header
    pub async fn get_block_header_hex(&self, hash: &QHash256) -> Result<String> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_text(&format!("block/{}/header", hash_str)).await
    }

    /// GET /block/:hash/status
    pub async fn get_block_status(&self, hash: &QHash256) -> Result<ElectrsBlockStatus> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_json(&format!("block/{}/status", hash_str)).await
    }

    /// GET /block/:hash/txs[/:start_index]
    pub async fn get_block_txs(
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
        self.get_json(&endpoint).await
    }

    /// GET /block/:hash/txids
    /// Returns all txids in block
    pub async fn get_block_txids(&self, hash: &QHash256) -> Result<Vec<QHash256>> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        let txids_hex: Vec<String> = self.get_json(&format!("block/{}/txids", hash_str)).await?;

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
    pub async fn get_block_txid_at_index(&self, hash: &QHash256, index: u32) -> Result<QHash256> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        let hex_str = self
            .get_text(&format!("block/{}/txid/{}", hash_str, index))
            .await?;

        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid txid hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    /// GET /block/:hash/raw
    pub async fn get_block_raw(&self, hash: &QHash256) -> Result<Vec<u8>> {
        let hash_str = Self::hash_to_hex_rpc(hash);
        self.get_bytes(&format!("block/{}/raw", hash_str)).await
    }

    /// GET /block-height/:height
    /// Returns the hash of the block at :height
    pub async fn get_block_hash(&self, height: u32) -> Result<QHash256> {
        let hex_str = self.get_text(&format!("block-height/{}", height)).await?;
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid block hash hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    /// GET /blocks[/:start_height]
    /// Returns 10 newest blocks
    pub async fn get_blocks(&self, start_height: Option<u32>) -> Result<Vec<ElectrsBlock>> {
        let endpoint = if let Some(h) = start_height {
            format!("blocks/{}", h)
        } else {
            "blocks".to_string()
        };
        self.get_json(&endpoint).await
    }

    /// GET /blocks/tip/height
    pub async fn get_block_height(&self) -> Result<u32> {
        let text = self.get_text("blocks/tip/height").await?;
        text.parse::<u32>().context("Failed to parse block height")
    }

    /// GET /blocks/tip/hash
    pub async fn get_tip_hash(&self) -> Result<QHash256> {
        let hex_str = self.get_text("blocks/tip/hash").await?;
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(hex_str.trim(), &mut bytes).context("Invalid tip hash hex")?;
        bytes.reverse(); // BE -> LE
        Ok(bytes)
    }

    // ========================================================================
    // General / Mempool Endpoints
    // ========================================================================

    /// GET /mempool
    pub async fn get_mempool_stats(&self) -> Result<ElectrsMempoolStats> {
        self.get_json("mempool").await
    }

    /// GET /mempool/txids
    pub async fn get_mempool_txids(&self) -> Result<Vec<QHash256>> {
        let txids_hex: Vec<String> = self.get_json("mempool/txids").await?;
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
    pub async fn get_mempool_recent(&self) -> Result<Vec<ElectrsMempoolRecent>> {
        self.get_json("mempool/recent").await
    }

    /// GET /fee-estimates
    pub async fn get_fee_estimates(&self) -> Result<HashMap<String, f64>> {
        self.get_json("fee-estimates").await
    }
}
