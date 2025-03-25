use doge_light_client::{core_data::QHash256, hash::merkle::merkle_proof::MerkleProofCore};
use hex::FromHexError;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;


#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, Eq, Hash, PartialOrd, Ord)]
pub struct WrappedHash256(#[serde_as(as = "serde_with::hex::Hex")] pub [u8; 32]);
impl Default for WrappedHash256 {
    fn default() -> Self {
        Self([0u8; 32])
    }
}


impl WrappedHash256 {
    pub const ZERO: Self = Self([0u8; 32]);
    pub fn from_hex_string(s: &str) -> Result<Self, FromHexError> {
        let bytes = hex::decode(s)?;
        assert_eq!(bytes.len(), 32);
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
    pub fn to_hex_string(&self) -> String {
        hex::encode(&self.0)
    }
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&x| x == 0)
    }
    pub fn reversed(&self) -> Self {
        WrappedHash256(core::array::from_fn(|i| self.0[31 - i]))
    }
}


pub fn q_proof_to_wq_proof(proof: &MerkleProofCore<QHash256>) -> MerkleProofCore<WrappedHash256> {
    MerkleProofCore {
        siblings: proof.siblings.iter().map(|x| WrappedHash256(*x)).collect(),
        index: proof.index,
        value: WrappedHash256(proof.value),
        root: WrappedHash256(proof.root),
    }
}