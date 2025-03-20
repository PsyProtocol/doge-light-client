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
pub enum DogeBridgeError {

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
    #[error("AuxPow version bits mismatch")]
    AuxPowVersionBitsMismatch = 601,
    #[error("AuxPow chain id mismatch")]
    AuxPowChainIdMismatch = 602,
    #[error("Difficulty bits mismatch")]
    DifficutlyBitsMismatch = 603,
    #[error("Standard proof of work check failed")]
    StandardPoWCheckFailed = 604,
    #[error("AuxPow parent block proof of work check failed")]
    AuxPowParentBlockPoWCheckFailed = 605,
    #[error("coinbase_branch.side_mask != 0, AuxPow is not a generate")]
    AuxPowCoinBaseBranchSideMaskNonZero = 606,
    #[error("Aux POW chain merkle branch too long")]
    AuxPowChainMerkleBranchTooLong = 607,
    #[error("Aux POW parent has our chain ID")]
    AuxPowParentHasOurChainId = 608,
    #[error("Aux POW merkle root incorrect")]
    IncorrectAuxPowMerkleRoot = 609,

    #[error("Aux POW coinbase has no inputs")]
    AuxPowCoinbaseNoInputs = 610,
    #[error("Aux POW missing chain merkle root in parent coinbase")]
    AuxPowCoinbaseMissingChainMerkleRoot = 611,
    #[error("MERGED_MINING_HEADER found twice in coinbase transaction input script")]
    MergedMiningHeaderFoundTwiceInCoinbase = 612,
    #[error("MERGED_MINING_HEADER not found at the beginning of the coinbase transaction input script")]
    MergedMiningHeaderNotFoundAtCoinbaseScriptStart = 613,
    #[error("chain merkle root starts too late in the coinbase transaction input script")]
    AuxPowChainMerkleRootTooLateInCoinbaseInputScript = 614,
    #[error("coinbase transaction input script is too short")]
    AuxPowCoinbaseTransactionInputScriptTooShort = 615,
    #[error("n_size in coinbase script does not correspond to the number of leaves of the merkle tree implictly defined by the blockchain branch hashes length")]
    AuxPowCoinbaseScriptInvalidNSize = 616,
    #[error("the sidemask provided in blockchain branch does not match the one computed from the coinbase transaction script")]
    AuxPowCoinbaseScriptInvalidSideMask = 617,
    #[error("InvalidReadableAccount")]
    InvalidReadableAccountExample = 618,
    #[error("PermissionViolation")]
    PermissionViolationExample = 619,
    #[error("NeedsSuccesfulAggregation")]
    NeedsSuccesfulAggregationExample = 620,
    #[error("MaxLastFeedIndexReached")]
    MaxLastFeedIndexReachedExample = 621,
    #[error("FeedIndexAlreadyInitialized")]
    FeedIndexAlreadyInitializedExample = 622,
    #[error("NoNeedToResize")]
    NoNeedToResizeExample = 623,



    /// start doge bridge runner stuff
    #[error("Attempted to fetch a Block at a height that is not stored in the cache (it is either too old or has not been processed yet)")]
    BlockNotInCache = 701,
    #[error("Attempted to modify an already finalized/confirmed block")]
    AttemptedToModifiyFinalizedBlock = 702,
    #[error("Insufficient block provided for rollback in the block data tracker")]
    InsufficientBlocksProvidedForRollback = 703,
    #[error("Attempted to insert a Block that already exists in the cache")]
    InsertBlockAlreadyInCache = 704,
    #[error("Attempted to append a block with a height that is not equal to the current tip + 1")]
    InsertBlockNotAtTip = 705,
    #[error("Parent block hash in block header does not match the current tip")]
    InvalidParentBlockHash = 706,
    #[error("AuxPow missing in aux pow block")]
    AuxPowMissing = 707,
    #[error("AuxPow found in non-aux pow block")]
    AuxPowNotExpected = 708,
    #[error("Block tip sync mismatch error")]
    BlockTipSyncMismatch = 709,
    #[error("The root of the Block tree after rollback doesn't match the known correct root")]
    RollbackBlockTreeRootMismatch = 710,
    #[error("The index of the block tree failed to rollback correctly")]
    RollbackBlockTreeIndexMismatch = 711,


    // start fixed append tree errors
    #[error("Cannot revert to index greater than or equal to current index")]
    RevertIndexTooHigh = 724,
    #[error("Not enough changed left siblings")]
    NotEnoughChangedLeftSiblings = 725,
    #[error("Revert index is not a prefix of current index")]
    RevertIndexNotPrefix = 726,
    #[error("Too many changed left siblings provided")]
    TooManyChangedLeftSiblings = 727,
}


#[cfg(feature = "solprogram")]
impl solana_program::program_error::PrintProgramError for DogeBridgeError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
#[cfg(feature = "solprogram")]
impl From<DogeBridgeError> for solana_program::program_error::ProgramError {
    fn from(e: DogeBridgeError) -> Self {
        solana_program::program_error::ProgramError::Custom(e as u32)
    }
}

#[cfg(feature = "solprogram")]
impl<T> solana_program::decode_error::DecodeError<T> for DogeBridgeError {
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


pub type QDogeResult<T> = Result<T, DogeBridgeError>;
