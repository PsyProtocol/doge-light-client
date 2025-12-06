use doge_light_client::common_types::QHash256;

use crate::bridge_state::bridge_header::PsyBridgeHeader;


#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy, Hash, Default)]
#[repr(C)]
pub struct PsyContractBridgeState {
    pub fee_collector_public_key: [u8; 32],
    pub pending_dtxo_bit_list_hash_stack: QHash256,
    pub pending_auto_processed_mint_hash_stack: QHash256,
    pub current_bridge_header: PsyBridgeHeader,
    pub fees_balance_sats: u64,
    pub required_confirmations: u32,
    pub total_blocks_processed: u32,
}
// START: Bridge Contract Serialization Logic
impl PsyContractBridgeState {
    #[cfg(feature = "serialize_bytemuck")]
    pub fn from_bytes(bytes: &[u8]) -> &Self {
        bytemuck::from_bytes(bytes)
    }
    #[cfg(feature = "serialize_bytemuck")]
    pub fn from_bytes_mut(bytes: &mut [u8]) -> &mut Self {
        bytemuck::from_bytes_mut(bytes)
    }
    #[cfg(feature = "serialize_bytemuck")]
    pub fn to_bytes_ref(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
// END: Bridge Contract Serialization Logic

// START: Bridge Contract Internal Function Logic
impl PsyContractBridgeState {
}
// END: Bridge Contract Internal Function Logic

// START: Bridge Contract Instruction Function Logic
impl PsyContractBridgeState {
}
// END: Bridge Contract Instruction Function Logic