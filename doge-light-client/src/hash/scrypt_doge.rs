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

use scrypt::Params;
pub fn scrypt_1024_1_1_256(data: &[u8]) -> [u8; 32] {
    // don't use this on solana as it requires too much compute/memory, instead prove it in a zkp + use the known pow hash feature
    let params = Params::new(10, 1, 1, 32).unwrap();
    let mut output = [0u8; 32];

    
    scrypt::scrypt(data, data, &params, &mut output).unwrap();
    output
}