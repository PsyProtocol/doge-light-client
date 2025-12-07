/*
Copyright (C) 2025 Zero Knowledge Labs Limited, Psy Protocol

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

"This software was created by Psy Protocol (https://psy.xyz)
with contributions from Carter Feldman (https://x.com/cmpeq)."
*/

use doge_light_client::common_types::{QHash160, QHash256};

use crate::wallet_common::signature::{CompressedPublicKey, PsyCompressedSecp256K1Signature};


pub trait Secp256K1WalletProvider {
    fn sign(
        &self,
        public_key: &CompressedPublicKey,
        message: QHash256,
    ) -> anyhow::Result<PsyCompressedSecp256K1Signature>;
    fn contains_public_key(&self, public_key: &CompressedPublicKey) -> bool;
    fn contains_p2pkh_address(&self, p2pkh_address: &QHash160) -> bool;
    fn get_public_key_for_p2pkh(&self, p2pkh: &QHash160) -> Option<CompressedPublicKey>;
    fn get_public_keys(&self) -> Vec<CompressedPublicKey>;
}