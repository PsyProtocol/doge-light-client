use psy_doge_wallet::secp256k1::signer::MemorySecp256K1Wallet;

pub struct DepositManyHelper {
    pub wallet_manager: MemorySecp256K1Wallet,
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Ok(())
}