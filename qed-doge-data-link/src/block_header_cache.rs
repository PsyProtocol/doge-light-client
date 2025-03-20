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

use std::collections::HashMap;

use doge_light_client::core_data::QDogeBlockHeader;
use serde::{Deserialize, Serialize};

use crate::{electrs_link::DogeLinkElectrsClient, traits::QDogeBlockHeaderFetcher};



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct BlockHeaderWithIndex {
    pub height: u32,
    pub block_header: QDogeBlockHeader,
}

#[derive(Debug, Clone)]
pub struct BlockHeaderFetcher {
    pub client: DogeLinkElectrsClient,
    pub store: HashMap<u32, QDogeBlockHeader>,
}

impl BlockHeaderFetcher {
    pub fn new(client: DogeLinkElectrsClient) -> Self {
        BlockHeaderFetcher {
            client,
            store: HashMap::new(),
        }
    }
    pub fn get_block_header(&mut self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        if let Some(header) = self.store.get(&height) {
            return Ok(header.clone());
        }
        let header = self.client.get_qd_block(height as u32)?.to_qdoge_block_header();
        self.store.insert(height, header.clone());
        Ok(header)
    }

    pub fn get_block_header_imm(&self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        if let Some(header) = self.store.get(&height) {
            return Ok(header.clone());
        }
        let header = self.client.get_qd_block(height as u32)?.to_qdoge_block_header();
        Ok(header)
    }

    pub fn get_block_headers(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        let mut block_headers = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            block_headers.push(self.get_block_header(*h)?);
        }
        Ok(block_headers)
    }
    pub fn get_block_headers_imm(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        let mut block_headers = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            block_headers.push(self.get_block_header_imm(*h)?);
        }
        Ok(block_headers)
    }
    pub fn to_block_headers(&self) -> Vec<BlockHeaderWithIndex> {
        let mut block_headers = self.store.iter().collect::<Vec<(&u32, &QDogeBlockHeader)>>();
        block_headers.sort_by(|a, b| a.0.cmp(b.0));
        block_headers.iter().map(|(h, b)| BlockHeaderWithIndex{height: **h, block_header: (*b).clone()}).collect()
    }
    pub fn save_block_headers_bin(&self, path: &str) -> anyhow::Result<()> {
        let block_headers = self.to_block_headers();
        let data = bincode::serialize(&block_headers)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    pub fn load_block_headers_bin(&mut self, path: &str) -> anyhow::Result<()> {
        let data = std::fs::read(path)?;
        let block_headers: Vec<BlockHeaderWithIndex> = bincode::deserialize(&data)?;
        for b in block_headers {
            self.store.insert(b.height, b.block_header);
        }
        Ok(())
    }
    pub fn save_block_headers(&self, path: &str) -> anyhow::Result<()> {
        let block_headers = self.to_block_headers();
        let data = serde_json::to_string(&block_headers)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    pub fn load_block_headers(&mut self, path: &str) -> anyhow::Result<()> {
        let data = std::fs::read_to_string(path)?;
        let block_headers: Vec<BlockHeaderWithIndex> = serde_json::from_str(&data)?;
        for b in block_headers {
            self.store.insert(b.height, b.block_header);
        }
        Ok(())
    }
}


impl QDogeBlockHeaderFetcher for BlockHeaderFetcher {
    fn get_qdoge_block_header(&self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        self.get_block_header_imm(height)
    }

    fn get_qdoge_block_headers(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        self.get_block_headers_imm(heights)
    }

    fn get_qdoge_block_header_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlockHeader> {
        self.get_block_header(height)
    }

    fn get_qdoge_block_headers_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>> {
        self.get_block_headers(heights)
    }
}