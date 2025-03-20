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


#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum DogeNetworkType {
    RegTest = 0,
    TestNet = 1,
    MainNet = 2,
}
impl DogeNetworkType {
    pub fn is_reg_test(&self) -> bool {
        self.eq(&Self::RegTest)
    }
    pub fn is_testnet(&self) -> bool {
        self.eq(&Self::RegTest)
    }
    pub fn is_mainnet(&self) -> bool {
        self.eq(&Self::RegTest)
    }
}



pub struct DogeNetworkParams {
    pub allow_min_difficulty_blocks: bool,
    pub pow_target_timespan: i64,
    pub pow_target_spacing: i64,
    pub pow_limit: u32,
    pub strict_chain_id: bool,
    pub aux_pow_chain_id: u32,
    pub min_valid_height: u32,
}

impl DogeNetworkParams {
    pub fn difficulty_adjustment_interval(&self) -> i64 {
        self.pow_target_timespan / self.pow_target_spacing
    }
}

