use std::collections::HashMap;



use doge_light_client::{common_types::{QHash160, QHash256}, hash::traits::DogeHashProvider};
use k256::ecdsa::signature::hazmat::PrehashSigner;

use crate::{traits::Secp256K1WalletProvider, wallet_common::signature::{CompressedPublicKey, CompressedPublicKeyToP2PKH, PsyCompressedSecp256K1Signature}};
pub trait SimpleSingleSigner {
    fn sign_message(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature>;
    fn get_compressed_public_key(&self) -> CompressedPublicKey;
}
pub struct SimpleSinglePublicKeySigner<T: Secp256K1WalletProvider> {
    pub wallet_provider: T,
    pub public_key: CompressedPublicKey,
}
impl<T: Secp256K1WalletProvider> SimpleSingleSigner for SimpleSinglePublicKeySigner<T> {
    fn sign_message(&self, message: QHash256) -> anyhow::Result<PsyCompressedSecp256K1Signature> {
        self.wallet_provider.sign(&self.public_key, message)
    }
    
    fn get_compressed_public_key(&self) -> CompressedPublicKey {
        self.public_key
    }
}
impl SimpleSinglePublicKeySigner<MemorySecp256K1Wallet> {
    pub fn new_insecure_memory_signer_with_private_key<Hasher: DogeHashProvider>(private_key: QHash256) -> anyhow::Result<Self> {
        let mut wallet = MemorySecp256K1Wallet::new();
        let public_key = wallet.add_private_key::<Hasher>(private_key)?;
        Ok(Self {
            wallet_provider: wallet,
            public_key,
        })
    }
}
#[derive(Debug, Clone)]
pub struct MemorySecp256K1Wallet {
    key_map: HashMap<CompressedPublicKey, k256::ecdsa::SigningKey>,
    p2pkh_key_map: HashMap<QHash160, CompressedPublicKey>,
}

impl Secp256K1WalletProvider for MemorySecp256K1Wallet {
    fn sign(
        &self,
        public_key: &CompressedPublicKey,
        message: QHash256,
    ) -> anyhow::Result<PsyCompressedSecp256K1Signature> {
        let private_key_result = self.key_map.get(public_key);
        if private_key_result.is_some() {
            let result: k256::ecdsa::Signature =
                private_key_result.unwrap().sign_prehash(&message)?;
            let mut rs_bytes = [0u8; 64];

            let r_bytes = result.r().to_bytes();
            let s_bytes = result.s().to_bytes();
            rs_bytes[0..32].copy_from_slice(&r_bytes);
            rs_bytes[32..64].copy_from_slice(&s_bytes);

            Ok(PsyCompressedSecp256K1Signature {
                public_key: public_key.0,
                signature: rs_bytes,
                message,
            })
        } else {
            anyhow::bail!("private key not found")
        }
    }


    fn contains_public_key(&self, public_key: &CompressedPublicKey) -> bool {
        self.key_map.contains_key(public_key)
    }

    fn get_public_keys(&self) -> Vec<CompressedPublicKey> {
        self.key_map.keys().cloned().collect()
    }

    fn contains_p2pkh_address(&self, p2pkh_address: &QHash160) -> bool {
        self.p2pkh_key_map.contains_key(p2pkh_address)
    }

    fn get_public_key_for_p2pkh(&self, p2pkh: &QHash160) -> Option<CompressedPublicKey> {
        self.p2pkh_key_map.get(p2pkh).cloned()
    }
}

impl MemorySecp256K1Wallet {
    pub fn new() -> Self {
        Self {
            key_map: HashMap::new(),
            p2pkh_key_map: HashMap::new(),
        }
    }
    pub fn add_private_key<Hasher: DogeHashProvider>(&mut self, private_key: QHash256) -> anyhow::Result<CompressedPublicKey> {
        let signing_key = k256::ecdsa::SigningKey::from_slice(&private_key)?;
        let public_key = signing_key
            .verifying_key()
            .to_encoded_point(true)
            .to_bytes();
        let mut compressed = [0u8; 33];
        if public_key.len() == 33 {
            compressed.copy_from_slice(&public_key);
        } else {
            anyhow::bail!("public key length is not 33")
        }
        let pub_compressed = CompressedPublicKey(compressed);
        let p2pkh = pub_compressed.to_p2pkh_address::<Hasher>();
        self.p2pkh_key_map.insert(p2pkh, pub_compressed);
        self.key_map.insert(pub_compressed, signing_key);
        Ok(pub_compressed)
    }
}
