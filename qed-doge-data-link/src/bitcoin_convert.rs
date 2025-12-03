use bitcoin::{block::SimpleHeader, hashes::Hash, Block};
use doge_light_client::{
    core_data::{
        QAuxPow, QDogeBlock, QDogeBlockHeader, QHash256, QMerkleBranch, QStandardBlockHeader,
    },
    doge::{coinbase_transaction::DogeAuxPowCoinbaseTransaction, transaction::BTCTransaction},
};

pub fn btc_block_header_to_qdoge(btc_block_header: &bitcoin::block::Header) -> QDogeBlockHeader {
    let header_bytes: Vec<u8> =
        bitcoin::consensus::encode::serialize::<SimpleHeader>(&btc_block_header.to_simple_header());

    let standard_header = QStandardBlockHeader::from_bytes(&header_bytes).unwrap();
    let aux_pow = if let Some(aux_data) = &btc_block_header.aux_data {
        Some(QAuxPow {
            coinbase_transaction: DogeAuxPowCoinbaseTransaction::from_bytes(
                &bitcoin::consensus::encode::serialize(&aux_data.coinbase_tx),
            )
            .unwrap(),
            block_hash: aux_data.block_hash.to_raw_hash().to_byte_array().into(),
            coinbase_branch: QMerkleBranch {
                side_mask: aux_data.coinbase_branch.side_mask,
                hashes: aux_data
                    .coinbase_branch
                    .hashes
                    .iter()
                    .map(|x| x.to_raw_hash().to_byte_array().into())
                    .collect::<Vec<QHash256>>(),
            },
            blockchain_branch: QMerkleBranch {
                side_mask: aux_data.blockchain_branch.side_mask,
                hashes: aux_data
                    .blockchain_branch
                    .hashes
                    .iter()
                    .map(|x| x.to_raw_hash().to_byte_array().into())
                    .collect::<Vec<QHash256>>(),
            },
            parent_block: QStandardBlockHeader::from_bytes(&bitcoin::consensus::encode::serialize(
                &aux_data.parent_block,
            ))
            .unwrap(),
        })
    } else {
        None
    };
    QDogeBlockHeader {
        header: standard_header,
        aux_pow,
    }
}

pub fn btc_block_to_qdoge(btc_block: &Block) -> anyhow::Result<QDogeBlock> {
    let txs = btc_block
        .txdata
        .iter()
        .map(|x| BTCTransaction::from_bytes(&bitcoin::consensus::encode::serialize(&x)))
        .collect::<anyhow::Result<Vec<BTCTransaction>>>()?;
    let header_bytes: Vec<u8> =
        bitcoin::consensus::encode::serialize::<SimpleHeader>(&btc_block.header.to_simple_header());

    let auxp = match &btc_block.header.aux_data {
        Some(ap) => Some(QAuxPow {
            coinbase_transaction: DogeAuxPowCoinbaseTransaction::from_bytes(
                &bitcoin::consensus::encode::serialize(&ap.coinbase_tx),
            )?,
            block_hash: ap.block_hash.to_raw_hash().to_byte_array().into(),
            coinbase_branch: QMerkleBranch {
                side_mask: ap.coinbase_branch.side_mask,
                hashes: ap
                    .coinbase_branch
                    .hashes
                    .iter()
                    .map(|x| x.to_raw_hash().to_byte_array().into())
                    .collect::<Vec<QHash256>>(),
            },
            blockchain_branch: QMerkleBranch {
                side_mask: ap.blockchain_branch.side_mask,
                hashes: ap
                    .blockchain_branch
                    .hashes
                    .iter()
                    .map(|x| x.to_raw_hash().to_byte_array().into())
                    .collect::<Vec<QHash256>>(),
            },
            parent_block: QStandardBlockHeader::from_bytes(
                &bitcoin::consensus::encode::serialize(&ap.parent_block),
            )?,
        }),
        None => None,
    };

    let qdb = QDogeBlock {
        header: QStandardBlockHeader::from_bytes(&header_bytes)?,
        transactions: txs,
        aux_pow: auxp,
    };
    Ok(qdb)
}

#[cfg(test)]
mod tests {
    use bitcoin::block::Header;
    use borsh::BorshDeserialize;

    use super::*;

    fn ensure_doge_block_header_round_trip_serialization_btc(raw_block_header_btc: &[u8]) {
        let bitcoin_version: Header =
            bitcoin::consensus::deserialize(raw_block_header_btc).unwrap();
        let txid = bitcoin_version.clone().aux_data.unwrap().coinbase_tx.txid();

        let qdoge_block_header = btc_block_header_to_qdoge(&bitcoin_version);
        let qdoge_block_header_bytes = qdoge_block_header.to_consensus_bytes();
        assert_eq!(raw_block_header_btc.to_vec(), qdoge_block_header_bytes);
    

        let auxin_q = qdoge_block_header.aux_pow.as_ref().unwrap();
        let txid_q = auxin_q.coinbase_transaction.get_hash();
        assert_eq!(txid.to_raw_hash().to_byte_array().to_vec(), txid_q.to_vec());
        let serialized_qdoge = borsh::to_vec(&qdoge_block_header).unwrap();
        let deserialized_qdoge = QDogeBlockHeader::try_from_slice(&serialized_qdoge).unwrap();
        assert_eq!(qdoge_block_header, deserialized_qdoge);
    }

    #[test]
    fn deserialize_doge_block_headers() {
        ensure_doge_block_header_round_trip_serialization_btc(
            &hex_literal::hex!("04016200eb7f3567726b4849cc2ec1802d59c5cb723e7766ed40536bc5af256617c9eb8cf645310d2537e390e5dd4ea6fd9203a94edcabc4ca1c4503b415e592880718592deeaf6601e7001a0000000001000000010000000000000000000000000000000000000000000000000000000000000000ffffffff440349b1290466afee2d2cfabe6d6dad36fbe4181a59c0bfa5d53d4ec5ad64c992b3d211b49f74928488c3fb7898754000000000000000042f4c502f08290082b807000000ffffffff0240be4025000000001976a91457757ed2d68143967543d7c58579c33e8984548888ac0000000000000000266a24aa21a9edb07efe56b5cb91f882ebd530fc2bae2f2763b1588c0dfe4cbc3aceb5253b7eb200000000f3a3c033da53859560794afa298333617c15c16744c5a56eb6c3371caf47f7c20169b7e4e942f4c395288a685be067a3c8755d95117321eed4333424be110a363400000000060000000000000000000000000000000000000000000000000000000000000000e2f61c3f71d1defd3fa999dfa36953755c690689799962b48bebd836974e8cf97d24db2bfa41474bfb2f877d688fac5faa5e10a2808cf9de307370b93352e548a8226bf3c1925bff4375f88ae8d44db411c4b02f21e40d6af0b2a12673327522177156a574e1cf0fae03b72c2943d7758979c74d1fe7a5a0fb0e86da77496330a551a6d0f21df11978aae5883eabaaae64a9126a57165cb037763c6f3d77f8203800000000000020b8feadb9d2d516cc67c9aebac010700eb4fe40a9fb98c69bd79692db1ed56de4b96df5cf2ef4dbc9af788f2411d2a2c085a27447105b63f05e9d24d910fb071d2deeaf66c5d276198360c71e"),
        );
        // auxpow with segwit from patrick
        ensure_doge_block_header_round_trip_serialization_btc(
            &hex_literal::hex!("0401620052e3397a263aa994b1cbade1df094843ee3d4414ec50f700df3e9fe13cde30cca0ceebf962d8757ec7cd8315adf9c5b75a9bf28a7dcda56de9393637f983cc1f0fc64e6583a8011a00000000020000000001010000000000000000000000000000000000000000000000000000000000000000ffffffff4403b05527fabe6d6d1c1e91303111f235329180ea89fb2976dd40568db66c3bae570568b007b87a2501000000000000005a554c55506f6f4c2d4c5443000005432cc40200ffffffff02f100a125000000001976a914f8394bea504520ac3ef09fd6a5adf70bede47dae88ac0000000000000000266a24aa21a9edde594137969fb1ab44095d93e452b01b20b9bcc477e8c913eba0e0645f39bf6a012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000b486809f957d7a60782215849b4228e87886b3f7fffbb8c10d0348a7aba810669aa2d585bb12a33e650844b5425c7a483278d63c3198f106b1db5205068c839d60eaefd3e1bf4144601dfbcbaf3dcd60d99d12dc3298b3df36150c745dd9763124002f88ee76619055019f7b1342638a6ad14eafb5204e59d5477a2e48f2fa5228699ae1a30c3c20c5fb669720e854b72190184652c31ead665049ddfd2f2ecef7c239308b9c51ee953c7fc616d74f3dedacfaeed7ea814a2f12555d1c2c9cf745f671f6a17b45f7f81fd005a461887540a1ca32b0e9cc8e2a700a2dfe08ad7d20dacf6e7f57719b5ee5f4911482a8ad08d649406819c565af927714e827f61ab3775c9b3080c18b38b6baff0b9a366da18682d275d16010538bb131c8ad53de8091f013fd342a7abab2d81e3a9034d848bb861dbf3ce3d6b706b63ebf4098242e110807f36604de297359be4ebf8ac927249ad2a1a9167b732cce2fb83775af100000000000000000000000020964243892e5af578b1afd1bce69ba7390aeb9c2858665135b03793d7f77950cd44f14269978a2c988e0c509d5dc6932b7d6685b9b423beddd633db567c5678e218c64e650592001a629d0b09"),
        );
    }
}
