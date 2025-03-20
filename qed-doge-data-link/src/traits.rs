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

use doge_light_client::core_data::{QDogeBlock, QDogeBlockHeader};

pub trait QDogeBlockFetcher {
    fn get_qdoge_block(&self, height: u32) -> anyhow::Result<QDogeBlock>;
    fn get_qdoge_blocks(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>>;
    fn get_qdoge_block_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlock>;
    fn get_qdoge_blocks_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlock>>;
}
pub trait QDogeBlockHeaderFetcher {
    fn get_qdoge_block_header(&self, height: u32) -> anyhow::Result<QDogeBlockHeader>;
    fn get_qdoge_block_headers(&self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>>;
    fn get_qdoge_block_header_cache(&mut self, height: u32) -> anyhow::Result<QDogeBlockHeader>;
    fn get_qdoge_block_headers_cache(&mut self, heights: &[u32]) -> anyhow::Result<Vec<QDogeBlockHeader>>;
}