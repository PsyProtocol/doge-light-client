
#[cfg(feature = "serialize_borsh")]
use borsh::{BorshSerialize, BorshDeserialize};
use doge_light_client::error::DogeBridgeError;
#[cfg(feature = "serialize_serde")]
use serde::{Serialize, Deserialize};

use num_derive::FromPrimitive;
use thiserror::Error;

#[cfg_attr(feature = "serialize_serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(BorshSerialize, BorshDeserialize))]
/// Errors that may be returned by the oracle program
#[derive(Clone, Debug, Eq, Error, PartialEq, Copy, FromPrimitive)]
pub enum ZKPVerifyError {
    #[error("Invalid bridge ZKP")]
    BridgeZKPError = 750,

    #[error("Invalid bridge ZKP provided as input")]
    InvalidBridgeInputZKP = 751,

    #[error("Invalid verifier key for bridge ZKP")]
    InvalidVerifierKeyForBridgeZKP = 752,

    #[error("Invalid public inputs for bridge ZKP")]
    InvalidPublicInputsForBridgeZKP = 753,
}


#[cfg(feature = "solprogram")]
impl solana_program::program_error::PrintProgramError for ZKPVerifyError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
#[cfg(feature = "solprogram")]
impl From<ZKPVerifyError> for solana_program::program_error::ProgramError {
    fn from(e: ZKPVerifyError) -> Self {
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}

#[cfg(feature = "solprogram")]
impl<T> solana_program::decode_error::DecodeError<T> for ZKPVerifyError {
    fn type_of() -> &'static str {
        "ZKP Verify Error"
    }
}

impl Into<DogeBridgeError> for ZKPVerifyError
{
    fn into(self) -> DogeBridgeError {
        match self {
            ZKPVerifyError::BridgeZKPError => DogeBridgeError::BridgeZKPError,
            ZKPVerifyError::InvalidBridgeInputZKP => DogeBridgeError::InvalidBridgeInputZKP,
            ZKPVerifyError::InvalidVerifierKeyForBridgeZKP => DogeBridgeError::InvalidVerifierKeyForBridgeZKP,
            ZKPVerifyError::InvalidPublicInputsForBridgeZKP => DogeBridgeError::InvalidPublicInputsForBridgeZKP,
        }
    }
}


pub trait TransitionZKPVerifier {
    fn verify_zkp(proof: &[u8], vkey: &[u8], public_inputs: &[u8]) -> Result<(), ZKPVerifyError>;
    fn verify_bridge_zkp(proof: &[u8], vkey: &[u8], public_inputs: &[u8]) -> Result<(), DogeBridgeError> {
        Self::verify_zkp(proof, vkey, public_inputs).map_err(|e| e.into())
    }
}