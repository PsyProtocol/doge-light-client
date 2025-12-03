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
use crate::doge::transaction::{BTCTransaction, BTCTransactionInput, BTCTransactionOutput};
use crate::hash::sha256::QBTCHash256Hasher;
use crate::hash::traits::BytesHasher;

use super::address::{AddressToBTCScript, BTCAddress160};
use super::varuint::{decode_varuint_partial, encode_varuint, varuint_size};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct DogeAuxPowCoinbaseTransaction {
    pub version: u32,
    pub inputs: Vec<DogeAuxPowCoinbaseTransactionInput>,
    pub outputs: Vec<BTCTransactionOutput>,
    pub locktime: u32,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd)]
pub struct DogeAuxPowCoinbaseTransactionInput {
    pub hash: QHash256,
    pub index: u32,
    pub script: Vec<u8>,
    pub sequence: u32,
    pub witness_element_count: u64,
    pub witness_raw: Vec<u8>,
}

/// The marker MUST be a 1-byte zero value: 0x00. (BIP-141)
const SEGWIT_MARKER: u8 = 0x00;
/// The flag MUST be a 1-byte non-zero value. Currently, 0x01 MUST be used. (BIP-141)
const SEGWIT_FLAG: u8 = 0x01;

const SEGWIT_MARKER_FLAG: [u8; 2] = [SEGWIT_MARKER, SEGWIT_FLAG];
impl DogeAuxPowCoinbaseTransaction {
    fn use_segwit_serialization(&self) -> bool {
        for input in &self.inputs {
            if !input.witness_raw.is_empty() {
                return true;
            }
        }
        // To avoid serialization ambiguity, no inputs means we use BIP141 serialization
        self.inputs.is_empty()
    }
    pub fn dummy() -> Self {
        Self {
            version: 2,
            inputs: vec![],
            outputs: vec![],
            locktime: 0,
        }
    }
    pub fn from_io(inputs: Vec<DogeAuxPowCoinbaseTransactionInput>, outputs: Vec<BTCTransactionOutput>) -> Self {
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
        for input in &self.inputs {
            if input.witness_raw.len() > 0 {
                return true;
            }
        }

        false
    }
    pub fn byte_length(&self, allow_witness: bool) -> usize {
        let use_segwit_serialization = allow_witness && self.use_segwit_serialization();
        // extra 2 bytes for segwit marker and flag
        let base: usize = if use_segwit_serialization { 10 } else { 8 };


        // for doge coin, we do not have witness for input, so we removed this from the BTC gadget

        base + varuint_size(self.inputs.len() as u64)
            + varuint_size(self.outputs.len() as u64)
            + self
                .inputs
                .iter()
                .map(|x| {
                    let base_size = 40 + x.script.len();
                    if use_segwit_serialization {
                        base_size + varuint_size(x.witness_element_count) + x.witness_raw.len()
                    } else {
                        base_size
                    }
                })
                .sum::<usize>()
            + self
                .outputs
                .iter()
                .map(|x| 8 + x.script.len())
                .sum::<usize>()
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
    pub fn get_tx_input_empty(&self) -> DogeAuxPowCoinbaseTransactionInput {
        DogeAuxPowCoinbaseTransactionInput {
            hash: self.get_hash(),
            index: 0,
            script: vec![],
            sequence: 4294967295,
            witness_element_count: 0,
            witness_raw: vec![],
        }
    }
    // version, locktime, output
    pub fn get_output_skip_decode(
        bytes: &[u8],
        start_offset: usize,
        output_index: usize,
    ) -> anyhow::Result<(u32, u32, BTCTransactionOutput)> {
        if bytes.len() - start_offset < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length, too small"));
        }
        let mut read_index = start_offset;

        let version = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;

        let mut inputs_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        read_index += inputs_len.1;
        let use_segwit_serialization = inputs_len.0 == 0;
        if use_segwit_serialization {
            // segwit serialization
            if bytes.len() - read_index < 2 {
                return Err(anyhow::anyhow!("Invalid bytes length for segwit marker and flag"));
            }
            if bytes[read_index] != SEGWIT_FLAG {
                return Err(anyhow::anyhow!("Invalid segwit marker"));
            }
            read_index += 1; // skip flag
            inputs_len = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
            read_index += inputs_len.1;
        }

        let inputs_size = inputs_len.0 as usize;
        //let mut inputs = vec![];
        for _ in 0..inputs_size {
            let (_script_size, offset) = DogeAuxPowCoinbaseTransactionInput::skip_decode(bytes, read_index)?;
            //inputs.push(input);
            // if any input has empty script, we use segwit serialization
            //use_segwit_serialization = use_segwit_serialization || script_size == 0;
            read_index = offset;
        }

        let outputs_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;

        read_index += outputs_len.1;
        let outputs_size = outputs_len.0 as usize;
        if output_index >= outputs_size {
            return Err(anyhow::anyhow!("Invalid output index"));
        }

        for _ in 0..output_index {
            let offset = BTCTransactionOutput::skip_decode(&bytes, read_index)?;
            read_index = offset;
        }
        let (output, offset) = BTCTransactionOutput::from_bytes(&bytes, read_index)?;
        read_index = offset;
        for _ in (output_index + 1)..outputs_size {
            let offset = BTCTransactionOutput::skip_decode(&bytes, offset)?;
            read_index = offset;
        }
        if use_segwit_serialization {
            for _ in 0..inputs_size {
                let offset = DogeAuxPowCoinbaseTransactionInput::skip_decode_witness(bytes, read_index)?;
                read_index = offset;
            }
        }

        let locktime = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;

        if read_index - start_offset != bytes.len() {
            return Err(anyhow::anyhow!(
                "Invalid bytes length, too large, {}, {}",
                read_index - start_offset,
                bytes.len()
            ));
        }

        Ok((version, locktime, output))
    }
    pub fn to_bytes(&self, allow_witness: bool) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(self.version.to_le_bytes());

        let use_segwit_serialization = allow_witness && self.use_segwit_serialization();
        if use_segwit_serialization {
            bytes.extend(&SEGWIT_MARKER_FLAG);
        }
        let inputs_len = encode_varuint(self.inputs.len() as u64); //serialize(&VarInt(self.inputs.len() as u64));
        bytes.extend(inputs_len);
        for input in &self.inputs {
            bytes.extend(input.to_bytes_standard());
        }
        let outputs_len = encode_varuint(self.outputs.len() as u64); //serialize(&VarInt(self.outputs.len() as u64));
        bytes.extend(outputs_len);
        for output in &self.outputs {
            bytes.extend(output.to_bytes());
        }
        if use_segwit_serialization {
            for input in &self.inputs {
                bytes.extend(input.witness_to_bytes());
            }
        }
        bytes.extend(self.locktime.to_le_bytes());
        bytes
    }
    pub fn from_bytes_offset(bytes: &[u8], offset: usize) -> anyhow::Result<(Self, usize)> {
        let total_remaining = bytes.len() - offset;
        if total_remaining < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        let version = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;

        let mut inputs_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        read_index += inputs_len.1;

        let use_segwit_serialization = inputs_len.0 == 0;

        if use_segwit_serialization {
            // segwit serialization
            if total_remaining < 32 + 4 + 4 + 1 + 2 {
                return Err(anyhow::anyhow!("Invalid bytes length for segwit marker and flag"));
            }
            if bytes[read_index] != SEGWIT_FLAG {
                return Err(anyhow::anyhow!("Invalid segwit marker"));
            }
            read_index += 1; // skip flag
            inputs_len = decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
            read_index += inputs_len.1;
        }




        let inputs_size = inputs_len.0 as usize;
        let mut inputs = vec![];
        for _ in 0..inputs_size {
            let (input, offset) = DogeAuxPowCoinbaseTransactionInput::standard_from_bytes(bytes, read_index)?;
            inputs.push(input);
            read_index = offset;
        }

        let outputs_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;

        read_index += outputs_len.1;

        let outputs_size = outputs_len.0 as usize;
        let mut outputs = vec![];
        for _ in 0..outputs_size {
            let (output, offset) = BTCTransactionOutput::from_bytes(&bytes, read_index)?;
            outputs.push(output);
            read_index = offset;
        }

        if use_segwit_serialization {
            for i in 0..inputs_size {
                let offset = inputs[i].augment_with_witness(bytes, read_index)?;
                read_index = offset;
            }
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
        QBTCHash256Hasher::hash_bytes(&self.to_bytes(false))
    }
    pub fn to_standard_transaction(&self) -> BTCTransaction {
        let inputs = self
            .inputs
            .iter()
            .map(|input| BTCTransactionInput {
                hash: input.hash,
                index: input.index,
                script: input.script.clone(),
                sequence: input.sequence,
            })
            .collect();
        BTCTransaction {
            version: self.version,
            inputs,
            outputs: self.outputs.clone(),
            locktime: self.locktime,
        }
    }
    pub fn into_standard_transaction(self) -> BTCTransaction {
        let inputs = self
            .inputs
            .into_iter()
            .map(|input| BTCTransactionInput {
                hash: input.hash,
                index: input.index,
                script: input.script,
                sequence: input.sequence,
            })
            .collect();
        BTCTransaction {
            version: self.version,
            inputs,
            outputs: self.outputs,
            locktime: self.locktime,
        }
    }
}
impl DogeAuxPowCoinbaseTransactionInput {
    pub fn to_bytes_standard(&self) -> Vec<u8> {
        let mut bytes = vec![];
        bytes.extend(&self.hash);
        bytes.extend(self.index.to_le_bytes());
        let len = encode_varuint(self.script.len() as u64); //serialize(&VarInt(self.script.len() as u64));
        bytes.extend(len);
        bytes.extend(&self.script);
        bytes.extend(self.sequence.to_le_bytes());
        bytes
    }
    pub fn witness_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(varuint_size(self.witness_element_count) + self.witness_raw.len());
        let len = encode_varuint(self.witness_element_count); //serialize(&VarInt(self.witness_element_count));
        bytes.extend(len);
        bytes.extend(&self.witness_raw);
        bytes
    }
    pub fn standard_from_bytes(bytes: &[u8], offset: usize) -> anyhow::Result<(Self, usize)> {
        if bytes.len() - offset < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        let hash_bytes: [u8; 32] = bytes[read_index..(read_index + 32)]
            .try_into()
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        let hash = hash_bytes;
        read_index += 32;
        let index = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;
        let script_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
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
                witness_element_count: 0,
                witness_raw: vec![],
            },
            read_index,
        ))
    }
    pub fn augment_with_witness(&mut self, bytes: &[u8], offset: usize) -> anyhow::Result<usize> {
        let (witness_element_count, witness_raw, read_index) = Self::witness_from_bytes(bytes, offset)?;
        self.witness_element_count = witness_element_count;
        self.witness_raw = witness_raw;
        Ok(read_index)
    }
    pub fn witness_from_bytes(bytes: &[u8], offset: usize) -> anyhow::Result<(u64, Vec<u8>, usize)> {
        let mut read_index = offset;
        let (witness_elem_count, witness_elem_count_size): (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        read_index += witness_elem_count_size;
        let mut witnesss_size = 0;
        let witness_start_index = read_index;
        
        for _ in 0..witness_elem_count {
            let elem_len: (u64, usize) =
                decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let full_size = elem_len.1 + (elem_len.0 as usize);
            if bytes.len() - read_index < full_size {
                return Err(anyhow::anyhow!("Invalid bytes length"));
            }
            read_index += full_size;
            witnesss_size += full_size;
        }


        let witness_raw = bytes[witness_start_index..(witness_start_index + witnesss_size)].to_vec();
        Ok((witness_elem_count, witness_raw, read_index))
    }
    pub fn skip_decode(bytes: &[u8], offset: usize) -> anyhow::Result<(u64, usize)> {
        if bytes.len() - offset < (32 + 4 + 4 + 1) {
            return Err(anyhow::anyhow!("Invalid bytes length"));
        }
        let mut read_index = offset;

        //let hash_bytes: [u8; 32] = bytes[read_index..(read_index + 32)].try_into().map_err(|e| anyhow::anyhow!("{:?}",e))?;
        //let hash = hash_bytes;
        read_index += 32;
        //let index = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;
        let script_len: (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        read_index += script_len.1;
        let script_size = script_len.0 as usize;

        //let script = bytes[read_index..(read_index + script_size)].to_vec();
        read_index += script_size;
        //let sequence = u32::from_le_bytes(bytes[read_index..(read_index + 4)].try_into().unwrap());
        read_index += 4;

        Ok((script_len.0, read_index - offset))
    }
    pub fn skip_decode_witness(bytes: &[u8], offset: usize) -> anyhow::Result<usize> {
        let mut read_index = offset;
        let (witness_element_count, witness_element_count_size): (u64, usize) =
            decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
        read_index += witness_element_count_size;

        for _ in 0..witness_element_count {
            let elem_len: (u64, usize) =
                decode_varuint_partial(&bytes[read_index..]).map_err(|e| anyhow::anyhow!("{:?}", e))?;
            let full_size = elem_len.1 + (elem_len.0 as usize);
            if bytes.len() - read_index < full_size {
                return Err(anyhow::anyhow!("Invalid bytes length"));
            }
            read_index += full_size;
        }

        Ok(read_index)
    }
}
impl Default for DogeAuxPowCoinbaseTransactionInput {
    fn default() -> Self {
        Self {
            hash: ([0u8; 32]),
            index: 0,
            script: vec![],
            sequence: 0,
            witness_element_count: 0,
            witness_raw: vec![],
        }
    }
}
#[cfg(test)]
mod tests {
    use bitcoin::{
        consensus::encode::{deserialize, serialize},
        Transaction,
    };

    use super::*;

    fn get_example_raw_txs() -> Vec<Vec<u8>> {
        vec![
            hex_literal::hex!("010000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff3603da1b0e00045503bd5704c7dd8a0d0ced13bb5785010800000000000a636b706f6f6c122f4e696e6a61506f6f6c2f5345475749542fffffffff02b4e5a212000000001976a914876fbb82ec05caa6af7a3b5e5a983aae6c6cc6d688ac0000000000000000266a24aa21a9edf91c46b49eb8a29089980f02ee6b57e7d63d33b18b4fddac2bcd7db2a39837040120000000000000000000000000000000000000000000000000000000000000000000000000").to_vec(),
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
    fn test_bitcoin_pkg_doge_aux_pow_tx() {
        let raw_txs = get_example_raw_txs();
        for rtx in raw_txs {
            let tx: Transaction = deserialize(&rtx).unwrap();
            let tx_bytes = serialize(&tx);
            assert_eq!(tx_bytes, rtx);

            let doge_aux_pow_tx = DogeAuxPowCoinbaseTransaction::from_bytes(&rtx).unwrap();
            let doge_aux_pow_tx_bytes = doge_aux_pow_tx.to_bytes(true);
            assert_eq!(doge_aux_pow_tx_bytes, rtx);
        }
    }
}
