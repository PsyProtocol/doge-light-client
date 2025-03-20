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

use std::str::FromStr;


use borsh::{BorshDeserialize, BorshSerialize};

use crate::{constants::{P2PKH_ADDRESS_CHECK58_VERSION, P2SH_ADDRESS_CHECK58_VERSION}, core_data::QHash160, hash::{ripemd160::QBTCHash160Hasher, traits::BytesHasher}};

use super::transaction::BTCTransactionOutput;


#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(
    PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd, Ord,
)]
pub enum BTCAddressType {
    P2PKH = 0,
    P2SH = 1,
}

#[cfg(feature = "serde")]

impl Serialize for BTCAddressType {
    fn serialize<S: serde::ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.to_u8(), serializer)
    }
}
#[cfg(feature = "serde")]

impl<'de> Deserialize<'de> for BTCAddressType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        let value = <u8 as Deserialize>::deserialize(deserializer)?;
        BTCAddressType::try_from(value).map_err(serde::de::Error::custom)
    }
}
impl BorshSerialize for BTCAddressType {
    fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
        BorshSerialize::serialize(&self.to_u8(), writer)
    }
}

impl BorshDeserialize for BTCAddressType {
    fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut res = [0u8; 1];
        reader.read_exact(&mut res)?;
       if res[0] == 0 {
            Ok(BTCAddressType::P2PKH)
        }else if res[0] == 1 {
            Ok(BTCAddressType::P2SH)
        }else{
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid BTCAddressType type",
            ))
        }
    }
}

impl BTCAddressType {
    pub fn to_u8(&self) -> u8 {
        *self as u8
    }
    pub fn to_version_byte(&self) -> u8 {
        match self {
            BTCAddressType::P2PKH => P2PKH_ADDRESS_CHECK58_VERSION,
            BTCAddressType::P2SH => P2SH_ADDRESS_CHECK58_VERSION,
        }
    }
    pub fn try_from_version_byte(version_byte: u8) -> anyhow::Result<Self> {
        match version_byte {
            P2PKH_ADDRESS_CHECK58_VERSION => Ok(BTCAddressType::P2PKH),
            P2SH_ADDRESS_CHECK58_VERSION => Ok(BTCAddressType::P2SH),
            _ => Err(anyhow::format_err!(
                "Invalid BTCAddressType version byte: {}",
                version_byte
            )),
        }
    }
}
impl From<BTCAddressType> for u8 {
    fn from(value: BTCAddressType) -> u8 {
        value as u8
    }
}
impl TryFrom<u8> for BTCAddressType {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BTCAddressType::P2PKH),
            1 => Ok(BTCAddressType::P2SH),
            _ => Err(anyhow::format_err!(
                "Invalid BTCAddressType type: {}",
                value
            )),
        }
    }
}

pub trait AddressToBTCScript {
    fn to_btc_script(&self) -> Vec<u8>;

    fn to_btc_output(&self, value: u64) -> BTCTransactionOutput {
        BTCTransactionOutput {
            value,
            script: self.to_btc_script(),
        }
    }
}


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash, PartialOrd, Ord)]
pub struct BTCAddress160 {
    pub address_type: BTCAddressType,
    pub address: QHash160,
}

impl BTCAddress160 {
    pub fn try_from_string(str: &str) -> anyhow::Result<Self> {
        let decoded = bs58::decode(str).with_check(None).into_vec().map_err(|e| anyhow::anyhow!("{:?}",e))?;
        if decoded.len() != 21 {
            return Err(anyhow::format_err!(
                "Invalid BTC address length: {}",
                decoded.len()
            ));
        }
        let address_type = BTCAddressType::try_from_version_byte(decoded[0])?;
        let mut hash_160_bytes = [0u8; 20];
        hash_160_bytes.copy_from_slice(&decoded[1..]);
        Ok(Self {
            address_type,
            address: (hash_160_bytes),
        })
    }
    pub fn from_p2pkh_key(key: &[u8; 33]) -> Self {
        Self {
            address_type: BTCAddressType::P2PKH,
            address: QBTCHash160Hasher::hash_bytes(key),
        }
    }
    pub fn new_p2pkh(address: QHash160) -> Self {
        Self {
            address_type: BTCAddressType::P2PKH,
            address,
        }
    }
    pub fn new_p2sh(address: QHash160) -> Self {
        Self {
            address_type: BTCAddressType::P2SH,
            address,
        }
    }
    pub fn to_address_string(&self) -> String {
        bs58::encode(self.address)
            .with_check_version(self.address_type.to_version_byte())
            .into_string()
    }
}

impl TryFrom<&str> for BTCAddress160 {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        BTCAddress160::try_from_string(value)
    }
}

#[cfg(feature = "bitcoin")]

impl TryFrom<bitcoin::Address> for BTCAddress160 {
    type Error = anyhow::Error;

    fn try_from(value: bitcoin::Address) -> Result<Self, Self::Error> {
        BTCAddress160::try_from_string(&value.to_string())
    }
}
/*
impl From<&BTCAddress160> for String {
    fn from(value: &BTCAddress160) -> Self {
        value.to_address_string()
    }
}
*/
impl ToString for BTCAddress160 {
    fn to_string(&self) -> String {
        self.to_address_string()
    }
}

impl FromStr for BTCAddress160 {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BTCAddress160::try_from_string(s)
    }
}
impl AddressToBTCScript for BTCAddress160 {
    fn to_btc_script(&self) -> Vec<u8> {
        match self.address_type {
            BTCAddressType::P2PKH => gen_p2pkh_script(&self.address).to_vec(),
            BTCAddressType::P2SH => gen_p2sh_script(&self.address).to_vec(),
        }
    }
}

pub fn gen_p2sh_script(hash: &QHash160) -> [u8; 23] {
    [
        0xa9, 0x14, hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6],
        hash[7], hash[8], hash[9], hash[10], hash[11], hash[12], hash[13],
        hash[14], hash[15], hash[16], hash[17], hash[18], hash[19], 0x87,
    ]
}

pub fn gen_p2pkh_script(hash: &QHash160) -> [u8; 25] {
    [
        0x76, 0xa9, 0x14, hash[0], hash[1], hash[2], hash[3], hash[4], hash[5],
        hash[6], hash[7], hash[8], hash[9], hash[10], hash[11], hash[12], hash[13],
        hash[14], hash[15], hash[16], hash[17], hash[18], hash[19], 0x88, 0xac,
    ]
}
