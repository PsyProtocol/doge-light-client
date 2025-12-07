use async_trait::async_trait;
use doge_light_client::common_types::QHash256;

#[async_trait]
pub trait DogecoinLinkProviderAsync {
    async fn get_block_raw(&self, block_hash: QHash256) -> anyhow::Result<Vec<u8>>;
}