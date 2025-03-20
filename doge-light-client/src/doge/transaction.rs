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

use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
//use bitcoin::consensus::{deserialize_partial, serialize};
//use bitcoin::VarInt;


use crate::core_data::QHash256;
use crate::hash::sha256::QBTCHash256Hasher;
use crate::hash::traits::BytesHasher;

use super::address::{AddressToBTCScript, BTCAddress160};
use super::varuint::{decode_varuint_partial, encode_varuint, varuint_size};


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct BTCTransaction {
    pub version: u32,
    pub inputs: Vec<BTCTransactionInput>,
    pub outputs: Vec<BTCTransactionOutput>,
    pub locktime: u32,
}





#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct BTCTransactionOutput {
    pub value: u64,
    pub script: Vec<u8>,
}




#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct BTCTransactionInputWithoutScript {
    pub hash: QHash256,
    pub index: u32,
    pub sequence: u32,
}


impl BTCTransactionInputWithoutScript {
    pub fn new(hash: QHash256, index: u32, sequence: u32) -> Self {
        Self {
            hash,
            index,
            sequence,
        }
    }
    pub fn new_simple(hash: QHash256, index: u32) -> Self {
        Self {
            hash,
            index,
            sequence: 0xffffffff,
        }
    }
}


#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct BTCTransactionInput {
    pub hash: QHash256,
    pub index: u32,
    pub script: Vec<u8>,
    pub sequence: u32,
    // TODO: implement witnesses for advanced transactions supported by bitcoin
    // pub witness: Vec<u8>,
}
impl BTCTransaction {
    pub fn dummy() -> Self {
        Self {
            version: 2,
            inputs: vec![],
            outputs: vec![],
            locktime: 0,
        }
    }
    pub fn from_partial(
        inputs: &[BTCTransactionInputWithoutScript],
        outputs: Vec<BTCTransactionOutput>,
    ) -> Self {
        Self {
            version: 2,
            inputs: inputs
                .into_iter()
                .map(|x| BTCTransactionInput {
                    hash: x.hash,
                    index: x.index,
                    script: vec![],
                    sequence: x.sequence,
                })
                .collect(),
            outputs: outputs,
            locktime: 0,
        }
    }
    pub fn from_io(inputs: Vec<BTCTransactionInput>, outputs: Vec<BTCTransactionOutput>) -> Self {
        Self {
            version: 2,
            inputs: inputs,
            outputs: outputs,
            locktime: 0,
        }
    }
    pub fn is_dummy(&self) -> bool {
        self.inputs.len() == 0 && self.outputs.len() == 0
    }
    pub fn get_vouts_for_address(&self, address: &BTCAddress160) -> Vec<u32> {
        let address_script = address.to_btc_script();
        self.outputs
            .iter()
            .enumerate()
            .filter_map(|(i, output)| {
                if output.script.eq(&address_script) {
                    Some(i as u32)
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn has_witnesses(&self) -> bool {
        // for doge coin we do not have witness for input, so we removed this from the BTC gadget

        /*
        for input in &self.inputs {
            if input.witness.len() > 0 {
                return true;
            }
        }
        */

        false
    }
    pub fn byte_length(&self, allow_witness: bool) -> usize {
        let has_witnesses = allow_witness && self.has_witnesses();
        let base: usize = if has_witnesses { 10 } else { 8 };

        // for doge coin, we do not have witness for input, so we removed this from the BTC gadget
        let witnesses_size = 0usize;

        base + varuint_size(self.inputs.len() as u64)
            + varuint_size(self.outputs.len() as u64)
            + self
                .inputs
                .iter()
                .map(|x| 40 + x.script.len())
                .sum::<usize>()
            + self
                .outputs
                .iter()
                .map(|x| 8 + x.script.len())
                .sum::<usize>()
            + witnesses_size
    }
    pub fn weight(&self) -> u64 {
        let base = self.byte_length(false) as u64;
        let total = self.byte_length(true) as u64;
        base * 3 + total
    }
    pub fn virtual_size(&self) -> u64 {
        let weight = self.weight();
        let extra = if (weight & 0b11) != 0 { 1u64 } else { 0u64 };
        (weight >> 2u64) + extra
    }
    /* 
    pub fn is_twm_spend_for_public_key(&self, expected_address: Hash160) -> bool {
        //let address = Hash160::from_bytes(&self.outputs[0].script[3..23]).unwrap();
        /*address == next_address
        && */
        
        self.outputs[0].script.len() == 23
            && self.inputs[0].script.len() > BLOCK_SCRIPT_LENGTH
            && btc_hash160(
                &self.inputs[0].script[(self.inputs[0].script.len() - BLOCK_SCRIPT_LENGTH)..],
            ) == expected_address
    }*/
    pub fn is_p2pkh(&self) -> bool {
        self.inputs.len() == 1
            && self.outputs.len() == 1
            && (self.inputs[0].script.len() == 106 || self.inputs[0].script.len() == 107)
    }
    pub fn get_tx_input_empty(&self) -> BTCTransactionInput {
        BTCTransactionInput {
            hash: self.get_hash(),
            index: 0,
            script: vec![],
            sequence: 4294967295,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.version.to_le_bytes());
        let inputs_len = encode_varuint(self.inputs.len() as u64);//serialize(&VarInt(self.inputs.len() as u64));
        bytes.extend(inputs_len);
        for input in &self.inputs {
            bytes.extend(input.to_bytes());
        }
        let outputs_len = encode_varuint(self.outputs.len() as u64);//serialize(&VarInt(self.outputs.len() as u64));
        bytes.extend(outputs_len);
        for output in &self.outputs {
            bytes.extend(output.to_bytes());
        }
        bytes.extend(self.locktime.to_le_bytes());
        bytes
    }
    pub fn from_bytes_offset(bytes: &[u8], offset: usize) -> anyhow::Result<(Self, usize)> {
        if bytes.len() - offset < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        let version = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;
        let inputs_len: (u64, usize) = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}",e))?;
        read_index += inputs_len.1;
        let inputs_size = inputs_len.0 as usize;
        let mut inputs = vec![];
        for _ in 0..inputs_size {
            let (input, offset) = BTCTransactionInput::from_bytes(bytes, read_index)?;
            inputs.push(input);
            read_index = offset;
        }
        let outputs_len: (u64, usize) = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}",e))?;
        read_index += outputs_len.1;
        let outputs_size = outputs_len.0 as usize;
        let mut outputs = vec![];
        for _ in 0..outputs_size {
            let (output, offset) = BTCTransactionOutput::from_bytes(bytes.to_vec(), read_index)?;
            outputs.push(output);
            read_index = offset;
        }
        let locktime = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        Ok((
            Self {
                version,
                inputs,
                outputs,
                locktime,
            },
            read_index + 4,
        ))
    }
    pub fn from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        let (tx, _) = Self::from_bytes_offset(bytes, 0)?;
        Ok(tx)
    }
    pub fn get_hash(&self) -> QHash256 {
        QBTCHash256Hasher::hash_bytes(&self.to_bytes())
    }
}
impl BTCTransactionInput {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(&self.hash);
        bytes.extend(self.index.to_le_bytes());
        let len = encode_varuint(self.script.len() as u64);//serialize(&VarInt(self.script.len() as u64));
        bytes.extend(len);
        bytes.extend(&self.script);
        bytes.extend(self.sequence.to_le_bytes());
        bytes
    }
    pub fn from_bytes(bytes: &[u8], offset: usize) -> anyhow::Result<(Self, usize)> {
        if bytes.len() - offset < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        let hash_bytes: [u8; 32] = bytes[read_index..(read_index + 32)].try_into().map_err(|e| anyhow::anyhow!("{:?}",e))?;
        let hash = hash_bytes;
        read_index += 32;
        let index = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;
        let script_len: (u64, usize) = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}",e))?;
        read_index += script_len.1;
        let script_size = script_len.0 as usize;

        let script = bytes[read_index..(read_index + script_size)].to_vec();
        read_index += script_size;
        let sequence = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;

        Ok((
            Self {
                hash,
                index,
                script,
                sequence,
            },
            read_index,
        ))
    }
}
impl Default for BTCTransactionInput {
    fn default() -> Self {
        Self {
            hash: ([0u8; 32]),
            index: 0,
            script: vec![],
            sequence: 0,
        }
    }
}

impl BTCTransactionOutput {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.value.to_le_bytes());
        let len = encode_varuint(self.script.len() as u64);//serialize(VarInt(self.script.len() as u64));
        bytes.extend(len);
        bytes.extend(&self.script);
        bytes
    }
    pub fn from_bytes(bytes: Vec<u8>, offset: usize) -> anyhow::Result<(Self, usize)> {
        if bytes.len() - offset < (8 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        let value = u64::from_le_bytes(bytes[read_index..(read_index + 8)].try_into().unwrap());
        read_index += 8;
        let script_len: (u64, usize) = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}",e))?;
        read_index += script_len.1;
        let script_size = script_len.0 as usize;

        let script = bytes[read_index..(read_index + script_size)].to_vec();
        read_index += script_size;

        Ok((Self { value, script }, read_index))
    }

    pub fn blank() -> Self {
        Self {
            value: 0xffffffffffffffffu64,
            script: vec![],
        }
    }
    pub fn is_p2pkh_output(&self) -> bool {
        self.script.len() == 25 && self.script[0] == 0x76 && self.script[1] == 0xa9 && self.script[2] == 0x14 && self.script[23] == 0x88 && self.script[24] == 0xac
    }
    pub fn is_p2sh_output(&self) -> bool {
        self.script.len() == 23 && self.script[0] == 0xa9 && self.script[1] == 0x14 && self.script[22] == 0x87
    }
    pub fn get_output_address(&self) -> anyhow::Result<BTCAddress160> {
        if self.is_p2pkh_output() {
            Ok(BTCAddress160::new_p2pkh(self.script[3..23].try_into().unwrap()))
        } else if self.is_p2sh_output() {
            Ok(BTCAddress160::new_p2sh(self.script[2..22].try_into().unwrap()))
        } else {
            Err(anyhow::anyhow!("could not find address in output script"))
        }
    }
}
impl Default for BTCTransactionOutput {
    fn default() -> Self {
        Self {
            value: 0,
            script: vec![],
        }
    }
}
#[cfg(test)]
mod tests {
    use bitcoin::{
        consensus::encode::{deserialize, serialize},
        Transaction,
    };

    use super::BTCTransaction;


    fn get_example_raw_txs() -> Vec<Vec<u8>> {
        vec![
            hex_literal::hex!("020000000142eedadeda5e79813b413d360b9e4a4dfe0f65159eb26eb5e3819954bd6bec4200000000fd1203305718d0a4c82f338c23ffdb184122fcd167159cee33024d243a1b656470e5595b5966eb2e18bdf384d1765beaedb372af30afff564fee031cfdb741e89884c80ebd2773ac14b2c6157b09caed45b39b051cf8b64ff43949f96aaff7935fe27e3b22303250ab2c76f8713b2d164828c7770ca02e9b2e8f13bbf64e0e21270e16ebf7a4446ac19bd8fa7d054ee31d56c2f2d999307520125401373dadedeacc198c175b814d548f780d336649e73ad96d7aeb443b01e22e73f808683f1eeb0e71575582ae4c500c8e4f5f9025c9a972b9970491740c0473465e81e64f32a51350bb054dc86a447999404a9e2c3533679a33034dcb310e88b9f797ffeb96230a055ac0f6d5ed4eb4ea316cd6b0a93d6f1ef714039d05944df9013008aa981e382121567aecaaf228e0b9722249cc4af36b98899a9990492b9858c9cfc7b9e1a1dc235d8342e5e5ff4d912c7c76a8201eee570455bbbd58923add8a280cbed0bcce549a2fdc780bba35621d37181b3d884c5057a7823a3e9b8e7d72389f4398707b78138d570fca0a9ae9a2f240ad3760ed8800f1400c516bd9a2c86725ff75b6ff09e87a71a5a7038d707ae5163a424cb44cc47c61d99fbac95835b38d8626c8268f4c500de5798a1ac6f3d4bfbd7f4ecb018fc5a1a35618c1543261d9edd51627faded3e81e6dd3560ad5632e6b746fc43ced61f5c8109ba680257343d49b9c55ab3c8b197cad346f4b214f90fb72fc4a1b1eb74c500e57bd51a2073f508cf82bb7305a648abddaf7e8053f6d004f7e8a39791ae1677e7af9291a2708f1ea2f4a83efc15bbde38f519624f962ac07bea41963a7b1836d4c53b5a4dbf2fbb3c1ce3e61765ed04c50447dcd68928fb58caf4d5250d973213b665d39cafb0da9414cabc8fb8341251086e3beec6c46a26b55cbe563010de2e71b2cdb4295c22734ed304a6fccc0bcb73980407863eebaa982a8067e97174d6d4c5079105ee3ee45b69efc35b4ab3f6dd6b3daa07c373ca3c26b2ce63a7002430aba4bb130f9cade132cf19632b02f44f98d7b50457b31f8ee73a4eee572a656da8b36910c1e4302f7731619bf64d9a78f7751926d6d6d6d6d6d51ffffffff01002f68590000000017a91400b6cf04571f8d62644b0fdfacf96538a18f3d4d8700000000").to_vec(),
        ]
    }
    #[test]
    fn test_bitcoin_pkg() {
        let raw_txs = get_example_raw_txs();
        for rtx in raw_txs {
            let tx: Transaction = deserialize(&rtx).unwrap();
            let tx_bytes = serialize(&tx);
            assert_eq!(tx_bytes, rtx);
        }
    }
    #[test]
    fn test_bitcoin_pkg_btc_tx() {
        let raw_txs = get_example_raw_txs();
        for rtx in raw_txs {
            let tx: Transaction = deserialize(&rtx).unwrap();
            let tx_bytes = serialize(&tx);
            assert_eq!(tx_bytes, rtx);

            let btc_tx = BTCTransaction::from_bytes(&rtx).unwrap();
            let btc_tx_bytes = btc_tx.to_bytes();
            assert_eq!(btc_tx_bytes, rtx);
        }
    }
}
