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

//! Error types

#[cfg(feature = "borsh")]
use borsh::{BorshSerialize, BorshDeserialize};
#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

use num_derive::FromPrimitive;
use thiserror::Error;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "borsh", derive(BorshSerialize, BorshDeserialize))]
/// Errors that may be returned by the oracle program
#[derive(Clone, Debug, Eq, Error, PartialEq, Copy, FromPrimitive)]
pub enum ClaimDogeBridgeHelperError {

    /// 0 - Error deserializing an account
    #[error("Error deserializing an account")]
    DeserializationError = 0,
    /// 1 - Error serializing an account
    #[error("Error serializing an account")]
    SerializationError = 1,
    /// 2 - Invalid program owner
    #[error("Invalid program owner. This likely mean the provided account does not exist")]
    InvalidProgramOwner = 2,
    /// 3 - Invalid PDA derivation
    #[error("Invalid PDA derivation")]
    InvalidPda = 3,
    /// 4 - Expected empty account
    #[error("Expected empty account")]
    ExpectedEmptyAccount = 4,
    /// 5 - Expected non empty account
    #[error("Expected non empty account")]
    ExpectedNonEmptyAccount = 5,
    /// 6 - Expected signer account
    #[error("Expected signer account")]
    ExpectedSignerAccount = 6,
    /// 7 - Expected writable account
    #[error("Expected writable account")]
    ExpectedWritableAccount = 7,
    /// 8 - Account mismatch
    #[error("Account mismatch")]
    AccountMismatch = 8,
    /// 9 - Invalid account key
    #[error("Invalid account key")]
    InvalidAccountKey = 9,
    /// 10 - Numerical overflow
    #[error("Numerical overflow")]
    NumericalOverflow = 10,


    /// Generic catch all error
    #[error("Unknown Error")]
    UnknownError = 600,

    /// start doge bridge demo stuff
    #[error("Invalid transaction proof V1 blob")]
    InvalidTransactionProofV1Blob = 601,
    #[error("Mismatched tx merkle roots")]
    MismatchedTxMerkleRoots = 602,
    #[error("Invalid proof transaction data found when decoding transaction")]
    InvalidProofTransactionData = 603,
    #[error("Invalid proof transaction version")]
    InvaildProofTransactionVersion = 604,


    #[error("Invalid proof transaction lock time")]
    InvaildProofTransactionLocktime = 605,

    
    #[error("Invalid proof transaction output")]
    InvalidProofTransactionOutput = 606,
    
    #[error("User already claimed this bridge transaction")]
    BridgeTransactionAlreadyClaimed = 607,

    #[error("Invalid user claim tree delta merkle proof (root does not match current state)")]
    MismatchedUserClaimDeltaMerkleProofOldRoot = 608,


    #[error("Missing block in cache")]
    BlockNotInCache = 609,

    #[error("Block not yet finalized")]
    BlockNotFinalized = 610,
}


#[cfg(feature = "solprogram")]
impl solana_program::program_error::PrintProgramError for ClaimDogeBridgeHelperError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
#[cfg(feature = "solprogram")]
impl From<ClaimDogeBridgeHelperError> for solana_program::program_error::ProgramError {
    fn from(e: ClaimDogeBridgeHelperError) -> Self {
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}

#[cfg(feature = "solprogram")]
impl<T> solana_program::decode_error::DecodeError<T> for ClaimDogeBridgeHelperError {
    fn type_of() -> &'static str {
        "Doge Bridge Error"
    }
}



#[macro_export]
macro_rules! doge_bail {
    ($err:expr $(,)?) => {
        return Err($err);
    };
}


pub type QClaimDogeResult<T> = Result<T, ClaimDogeBridgeHelperError>;
