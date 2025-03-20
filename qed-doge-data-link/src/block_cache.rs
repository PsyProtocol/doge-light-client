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

use doge_light_client::core_data::QDogeBlock;
use serde::{Deserialize, Serialize};

use crate::{electrs_link::DogeLinkElectrsClient, traits::QDogeBlockFetcher};



#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlockWithIndex {
    pub height: u32,
    pub block: QDogeBlock,
}
#[derive(Debug, Clone)]
pub struct BlockFetcher {
    pub client: DogeLinkElectrsClient,
    pub store: HashMap<u32, QDogeBlock>,
}

impl BlockFetcher {
    pub fn new(client: DogeLinkElectrsClient) -> Self {
        BlockFetcher {
            client,
            store: HashMap::new(),
        }
    }
    pub fn get_block(&mut self, height: u32) -> anyhow::Result<QDogeBlock> {
        if let Some(block) = self.store.get(&height) {
            return Ok(block.clone());
        }
        let block = self.client.get_qdoge_block(height as u32)?;
        self.store.insert(height, block.clone());
        Ok(block)
    }

    pub fn get_blocks(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        let mut blocks = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            blocks.push(self.get_block(*h)?);
        }
        Ok(blocks)
    }
    pub fn get_block_imm(&self, height: u32) -> anyhow::Result<QDogeBlock> {
        if let Some(block) = self.store.get(&height) {
            return Ok(block.clone());
        }
        let block = self.client.get_qdoge_block(height as u32)?;
        Ok(block)
    }

    pub fn get_blocks_imm(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        let mut blocks = Vec::with_capacity(heights.len());
        for h in heights.iter(){
            blocks.push(self.get_block_imm(*h)?);
        }
        Ok(blocks)
    }
    pub fn to_blocks(&self) -> Vec<BlockWithIndex> {
        let mut blocks = self.store.iter().collect::<Vec<(&u32, &QDogeBlock)>>();
        blocks.sort_by(|a, b| a.0.cmp(b.0));
        blocks.iter().map(|(h, b)| BlockWithIndex{height: **h, block: (*b).clone()}).collect()
    }
    pub fn save_blocks(&self, path: &str) -> anyhow::Result<()> {
        let blocks = self.to_blocks();
        let data = serde_json::to_string(&blocks)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    pub fn load_blocks(&mut self, path: &str) -> anyhow::Result<()> {
        let data = std::fs::read_to_string(path)?;
        let blocks: Vec<BlockWithIndex> = serde_json::from_str(&data)?;
        for b in blocks {
            self.store.insert(b.height, b.block);
        }
        Ok(())
    }
    pub fn save_blocks_bin(&self, path: &str) -> anyhow::Result<()> {
        let blocks = self.to_blocks();
        let data = bincode::serialize(&blocks)?;
        std::fs::write(path, data)?;
        Ok(())
    }
    pub fn load_blocks_bin(&mut self, path: &str) -> anyhow::Result<()> {
        let data = std::fs::read(path)?;
        let blocks: Vec<BlockWithIndex> = bincode::deserialize(&data)?;
        for b in blocks {
            self.store.insert(b.height, b.block);
        }
        Ok(())
    }
}


impl QDogeBlockFetcher for BlockFetcher {
    fn get_qdoge_block(&self, height: u32) -> anyhow::Result<QDogeBlock> {
        self.get_block_imm(height)
    }

    fn get_qdoge_blocks(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        self.get_blocks_imm(heights)
    }

    fn get_qdoge_block_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlock> {
        self.get_block(height)
    }

    fn get_qdoge_blocks_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>> {
        self.get_blocks(heights)
    }
}