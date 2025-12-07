use bitcoin::Block;
use doge_light_client::{core_data::{QDogeBlock, QDogeBlockHeader}, doge::transaction::BTCTransaction};
use psy_doge_data_link::{link_async::DogeElectrsClient, link_common::bitcoin_convert::btc_block_to_qdoge};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = DogeElectrsClient::new("https://doge-electrs-demo.qed.me");

    let latest = client.get_blocks(None).await?;
    let raw_block: Vec<u8> = client.get_block_raw(&latest[0].id.reversed().0).await?;
    let btc_block: Block = bitcoin::consensus::deserialize(&raw_block)?;
    let qdoge_block = QDogeBlock::from_consensus_bytes(&raw_block)?;
    let btc_bytes = bitcoin::consensus::serialize(&btc_block);
    if btc_bytes != raw_block {
        anyhow::bail!("BTC block serialization does not match raw bytes from electrs!");
    } else {
        println!("BTC block serialization matches raw bytes from electrs.");
    }
    let qdoge_bytes = qdoge_block.to_consensus_bytes();
    if qdoge_bytes != raw_block {
        anyhow::bail!("QDoge block serialization does not match raw bytes from electrs!");
    } else {
        println!("QDoge block serialization matches raw bytes from electrs.");
    }
    let btc_header = btc_block.header.clone();
    let qdoge_header = QDogeBlockHeader::from_consensus_bytes(&bitcoin::consensus::serialize(&btc_header))?;
    let qdoge_header_bytes = qdoge_header.to_consensus_bytes();
    let btc_header_bytes = bitcoin::consensus::serialize(&btc_header);
    if qdoge_header_bytes != btc_header_bytes {
        anyhow::bail!("QDoge block header serialization does not match BTC block header serialization!");
    } else {
        println!("QDoge block header serialization matches BTC block header serialization.");
    }
    let converted = btc_block_to_qdoge(&btc_block)?;
    if converted.header != qdoge_block.header {
        anyhow::bail!("Converted QDoge block header does not match original QDoge block header!");
    } else {
        println!("Converted QDoge block header matches original QDoge block header.");
    }
    if converted.transactions.len() != qdoge_block.transactions.len() {
        anyhow::bail!("Converted QDoge block transactions count does not match original QDoge block transactions count!");
    } else {
        println!("Converted QDoge block transactions count matches original QDoge block transactions count.");
    }
    for (qtx, btx) in converted.transactions.iter().zip(btc_block.txdata.iter()) {
        let q_bytes = qtx.to_bytes();
        let b_bytes = bitcoin::consensus::serialize(btx);
        if q_bytes != b_bytes {
            anyhow::bail!("Transaction serialization mismatch!");
        }
        let qtx_from_bytes = BTCTransaction::from_bytes(&q_bytes)?;
        if qtx_from_bytes != *qtx {
            anyhow::bail!("Transaction deserialization mismatch!");
        }
        let round_trip = qtx_from_bytes.to_bytes();
        if round_trip != q_bytes {
            anyhow::bail!("Transaction round-trip serialization mismatch!");
        }
        println!("Transaction serialization/deserialization matches for txid {}", btx.txid());

    }




    Ok(())
}

/*
thread 'main' (228455047) panicked at /Users/carter/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-0.2.25/src/time/driver/handle.rs:24:32:
there is no timer running, must be called from the context of a Tokio 0.2.x runtime
stack backtrace:

*/