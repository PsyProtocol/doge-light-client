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

pub fn varuint_size(value: u64) -> usize {
    if value < 0xfd {
        1
    } else if value <= 0xffff {
        3
    } else if value <= 0xffffffff {
        5
    } else {
        9
    }
}

pub fn encode_varuint(value: u64) -> Vec<u8> {
    if value < 0xfd {
        vec![value as u8]
    } else if value <= 0xffffu64 {
        let mut v = vec![0xfd];
        v.extend_from_slice(&(value as u16).to_le_bytes());
        v
    } else if value <= 0xffffffffu64 {
        let mut v = vec![0xfe];
        v.extend_from_slice(&(value as u32).to_le_bytes());
        v
    } else {
        let mut v = vec![0xff];
        v.extend_from_slice(&value.to_le_bytes());
        v
    }
}
#[derive(Debug, Clone)]
pub struct VaruintDecodingError;

impl core::fmt::Display for VaruintDecodingError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "tried to decode malformed bytes into varuint")
    }
}

pub fn decode_varuint(data: &[u8]) -> Result<u64, VaruintDecodingError> {
    if data.is_empty() {
        return Err(VaruintDecodingError);
    }
    let first_byte = data[0];
    if first_byte < 0xfd {
        Ok(first_byte as u64)
    } else if first_byte == 0xfd {
        if data.len() < 3 {
            return Err(VaruintDecodingError);
        }
        Ok(u64::from_le_bytes([data[1], data[2], 0, 0, 0, 0, 0, 0]))
    } else if first_byte == 0xfe {
        if data.len() < 5 {
            return Err(VaruintDecodingError);
        }
        Ok(u64::from_le_bytes([
            data[1], data[2], data[3], data[4], 0, 0, 0, 0,
        ]))
    } else {
        if data.len() < 9 {
            return Err(VaruintDecodingError);
        }
        Ok(u64::from_le_bytes([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]))
    }
}

pub fn decode_varuint_partial(data: &[u8]) -> Result<(u64, usize), VaruintDecodingError> {
    if data.is_empty() {
        return Err(VaruintDecodingError);
    }
    let first_byte = data[0];
    if first_byte < 0xfd {
        Ok((first_byte as u64, 1))
    } else if first_byte == 0xfd {
        if data.len() < 3 {
            return Err(VaruintDecodingError);
        }
        Ok((u64::from_le_bytes([data[1], data[2], 0, 0, 0, 0, 0, 0]),3))
    } else if first_byte == 0xfe {
        if data.len() < 5 {
            return Err(VaruintDecodingError);
        }
        Ok((u64::from_le_bytes([
            data[1], data[2], data[3], data[4], 0, 0, 0, 0,
        ]),5))
    } else {
        if data.len() < 9 {
            return Err(VaruintDecodingError);
        }
        Ok((u64::from_le_bytes([
            data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
        ]),9))
    }
}
