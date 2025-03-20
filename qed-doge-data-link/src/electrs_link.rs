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

use std::time::Duration;

use bitcoin::{block::SimpleHeader, hashes::Hash, Block};
use doge_light_client::{core_data::{QAuxPow, QDogeBlock, QDogeBlockHeader, QHash256, QMerkleBranch, QStandardBlockHeader}, doge::transaction::BTCTransaction, network_params::DogeNetworkType};
use serde::de::DeserializeOwned;
use ureq::Agent;

use crate::traits::{QDogeBlockFetcher, QDogeBlockHeaderFetcher};


#[derive(Debug, Clone)]
pub struct DogeLinkElectrsClient {
    electrs_url: String,
    pub network: DogeNetworkType,
    electrs_client: ureq::Agent,
}


fn btc_block_to_qdoge(btc_block: &Block) -> anyhow::Result<QDogeBlock> {

    let txs = btc_block.txdata.iter().map(|x|BTCTransaction::from_bytes(&bitcoin::consensus::encode::serialize(&x))).collect::<anyhow::Result<Vec<BTCTransaction>>>()?;
    let header_bytes: Vec<u8> = bitcoin::consensus::encode::serialize::<SimpleHeader>(&btc_block.header.to_simple_header());

    




    let auxp = match &btc_block.header.aux_data {
        Some(ap) => {
            Some(QAuxPow {
                coinbase_transaction: BTCTransaction::from_bytes(&bitcoin::consensus::encode::serialize(&ap.coinbase_tx))?,
                block_hash: ap.block_hash.to_raw_hash().to_byte_array().into(),
                coinbase_branch: QMerkleBranch {
                    side_mask: ap.coinbase_branch.side_mask,
                    hashes: ap.coinbase_branch.hashes.iter().map(|x|x.to_raw_hash().to_byte_array().into()).collect::<Vec<QHash256>>(),
                },
                blockchain_branch: QMerkleBranch {
                    side_mask: ap.blockchain_branch.side_mask,
                    hashes: ap.blockchain_branch.hashes.iter().map(|x|x.to_raw_hash().to_byte_array().into()).collect::<Vec<QHash256>>(),
                },
                parent_block: QStandardBlockHeader::from_bytes(&bitcoin::consensus::encode::serialize(&ap.parent_block))?,
            })
        },
        None => None,
    };

    let qdb = QDogeBlock {
        header: QStandardBlockHeader::from_bytes(&header_bytes)?,
        transactions: txs,
        aux_pow: auxp,
    };
    Ok(qdb)
}

impl DogeLinkElectrsClient {
    pub fn new(electrs_url: String, network: DogeNetworkType) -> Self {
        let electrs_client =  Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .build().into();
        DogeLinkElectrsClient {
            electrs_url,
            network,
            electrs_client,
        }
    }
    pub fn get_json<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let url = format!("{}/{}", self.electrs_url, path);
        let response = self.electrs_client.get(&url).call()?.body_mut().read_to_string()?;
        serde_json::from_str(&response).map_err(|e| anyhow::anyhow!("{:?}", e))
    }
    pub fn get_bytes(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        let url = format!("{}/{}", self.electrs_url, path);
        let res = self.electrs_client.get(&url).call()?.body_mut().read_to_vec()?;
        Ok(res)
    }
    pub fn get_text(&self, path: &str) -> anyhow::Result<String> {
        let url = format!("{}/{}", self.electrs_url, path);
        let res = self.electrs_client.get(&url).call()?.body_mut().read_to_string()?;
        Ok(res)
    }

    pub fn get_block_height(&self) -> anyhow::Result<u32> {
        let height: u32 = self.get_json("blocks/tip/height")?;
        Ok(height)
    }
    pub fn get_block_hash(&self, height: u32) -> anyhow::Result<QHash256> {
        let hash_txt = self.get_text(&format!("block-height/{}", height))?;
        let mut hash = [0u8; 32];
        hex::decode_to_slice(hash_txt, &mut hash)?;
        Ok(hash)
    }
    pub fn get_block(&self, height: u32) -> anyhow::Result<Block> {
        let hash_txt = self.get_text(&format!("block-height/{}", height))?;
        let block_data = self.get_bytes(&format!("block/{}/raw", hash_txt))?;
        let btc_block: Block = bitcoin::consensus::encode::deserialize(&block_data)?;

        //let bh  = bitcoin::consensus::encode::serialize(&btc_block.header);
        //println!("bh_len: {}", bh.len());

        Ok(btc_block)

    }
    pub fn get_blocks(&self, heights: &[u32]) -> anyhow::Result<Vec<Block>> {
        let mut blocks = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            blocks.push(self.get_block(*h)?);
        }
        Ok(blocks)
    }
    pub fn get_qd_block(&self, height: u32) -> anyhow::Result<QDogeBlock> {
        btc_block_to_qdoge(&self.get_block(height)?)
    }
    pub fn get_qd_blocks(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        let mut blocks = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            blocks.push(self.get_qdoge_block(*h)?);
        }
        Ok(blocks)
    }
}



impl QDogeBlockFetcher for DogeLinkElectrsClient {
    fn get_qdoge_block(&self, height: u32) -> anyhow::Result<QDogeBlock> {
        self.get_qd_block(height)
    }

    fn get_qdoge_blocks(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        self.get_qd_blocks(heights)
    }

    fn get_qdoge_block_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlock> {
        self.get_qd_block(height)
    }

    fn get_qdoge_blocks_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        self.get_qd_blocks(heights)
    }
}
impl QDogeBlockHeaderFetcher for DogeLinkElectrsClient {
    fn get_qdoge_block_header(&self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        Ok(self.get_qd_block(height)?.to_qdoge_block_header())
    }

    fn get_qdoge_block_headers(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        Ok(self.get_qd_blocks(heights)?.iter().map(|x|x.to_qdoge_block_header()).collect())
    }

    fn get_qdoge_block_header_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        Ok(self.get_qd_block(height)?.to_qdoge_block_header())
    }

    fn get_qdoge_block_headers_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        Ok(self.get_qd_blocks(heights)?.iter().map(|x|x.to_qdoge_block_header()).collect())
    }
}