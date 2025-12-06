/*
Copyright (C) 2025 Zero Knowledge Labs Limited, Psy Protocol

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

"This software was created by Psy Protocol (https://Psy.xyz)
with contributions from Carter Feldman (https://x.com/cmpeq)."
*/

#[cfg(feature = "solprogram")]
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use doge_light_client::hash::sha256_impl::hash_impl_sha256_bytes;

#[cfg(not(feature = "solprogram"))]
use crate::utils::program_error::{ProgramError, ProgramResult};


// ============================================================================
// Constants & Structs
// ============================================================================

const MAX_PENDING_MINTS_PER_GROUP: usize = 24;
const MAX_PENDING_MINTS_PER_GROUP_U16: u16 = MAX_PENDING_MINTS_PER_GROUP as u16;

#[cfg(feature = "solprogram")]
const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;

#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy, Hash, Default)]
#[repr(C)]
pub struct PendingMint {
    pub recipient: [u8; 32],
    pub amount: u64,
}
const PENDING_MINT_SIZE: usize = std::mem::size_of::<PendingMint>();
const _RUN_TIME_ASSERT_PENDING_MINT_SIZE: () = assert!(PENDING_MINT_SIZE == 40);

#[cfg_attr(feature = "serialize_serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serialize_borsh", derive(borsh::BorshSerialize, borsh::BorshDeserialize))]
#[cfg_attr(feature = "serialize_speedy", derive(speedy::Readable, speedy::Writable))]
#[cfg_attr(feature = "serialize_bytemuck", derive(bytemuck::Pod, bytemuck::Zeroable))]
#[derive(PartialEq, Clone, Debug, Eq, Ord, PartialOrd, Copy, Hash, Default)]
#[repr(C)]
pub struct DataContractStateHeader {
    // Offset 0
    pub authorized_locker_public_key: [u8; 32],
    // Offset 32
    pub is_locked: u8,
    // Offset 33
    pub mode: u8,
    // Offset 34 (Aligned to 2)
    pub pending_mint_groups_count: u16,
    // Offset 36
    pub pending_mints_initialized: u16,
    // Offset 38
    pub pending_mints_count: u16,
}
const HEADER_SIZE: usize = std::mem::size_of::<DataContractStateHeader>();
const _RUN_TIME_ASSERT_HEADER_SIZE: () = assert!(HEADER_SIZE == 40);
// ============================================================================
// Helpers
// ============================================================================
#[cfg(feature = "solprogram")]
pub fn transfer_lamports_from_pdas<'a>(
    from: &AccountInfo<'a>,
    to: &AccountInfo<'a>,
    lamports: u64,
) -> ProgramResult {
    let mut from_lamports = from.try_borrow_mut_lamports()?;
    let mut to_lamports = to.try_borrow_mut_lamports()?;

    **from_lamports = from_lamports
        .checked_sub(lamports)
        .ok_or(ProgramError::InsufficientFunds)?;

    **to_lamports = to_lamports
        .checked_add(lamports)
        .ok_or(ProgramError::InvalidAccountData)?;

    Ok(())
}
#[cfg(feature = "solprogram")]
#[inline(always)]
pub fn realloc_account<'a>(
    target_account: &AccountInfo<'a>,
    funding_account: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
    refund: bool,
) -> ProgramResult {
    let rent = Rent::get()?;
    let old_minimum_balance = rent.minimum_balance(target_account.data_len());
    let new_minimum_balance = rent.minimum_balance(new_size);

    if new_minimum_balance > old_minimum_balance {
        let lamports_diff = new_minimum_balance - old_minimum_balance;
        invoke(
            &system_instruction::transfer(funding_account.key, target_account.key, lamports_diff),
            &[
                funding_account.clone(),
                target_account.clone(),
                system_program.clone(),
            ],
        )?;
    } else if refund && old_minimum_balance > new_minimum_balance {
        let lamports_diff = old_minimum_balance - new_minimum_balance;
        transfer_lamports_from_pdas(target_account, funding_account, lamports_diff)?;
    }

    target_account.realloc(new_size, false)
}

// ============================================================================
// Impl
// ============================================================================

impl DataContractStateHeader {
    pub fn setup(&mut self, authorized_locker_public_key: [u8; 32]) -> ProgramResult {
        if self.authorized_locker_public_key != [0u8; 32] {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        self.authorized_locker_public_key = authorized_locker_public_key;
        self.is_locked = 0;
        self.mode = 0;
        self.pending_mint_groups_count = 0;
        self.pending_mints_initialized = 0;
        self.pending_mints_count = 0;
        Ok(())
    }

    pub fn reinit(&mut self, pending_mints_count: u16) -> ProgramResult {
        if self.is_locked != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        self.pending_mints_count = pending_mints_count;
        self.pending_mint_groups_count = (pending_mints_count + MAX_PENDING_MINTS_PER_GROUP_U16
            - 1)
            / MAX_PENDING_MINTS_PER_GROUP_U16;
        self.pending_mints_initialized = 0;
        Ok(())
    }

    pub fn lock(&mut self, locker_public_key: [u8; 32]) -> ProgramResult {
        if self.is_locked != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        if self.pending_mints_initialized != self.pending_mints_count {
            return Err(ProgramError::InvalidAccountData);
        }
        if locker_public_key != self.authorized_locker_public_key {
            return Err(ProgramError::IllegalOwner);
        }
        self.is_locked = 1;
        Ok(())
    }

    pub fn unlock(&mut self, locker_public_key: [u8; 32]) -> ProgramResult {
        if self.is_locked == 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        if locker_public_key != self.authorized_locker_public_key {
            return Err(ProgramError::IllegalOwner);
        }
        self.is_locked = 0;
        self.pending_mint_groups_count = 0;
        self.pending_mints_count = 0;
        self.pending_mints_initialized = 0;
        Ok(())
    }

    pub fn get_total_buffer_size(&self) -> usize {
        HEADER_SIZE
            + 32 * (self.pending_mint_groups_count as usize)
            + (self.pending_mints_count as usize) * PENDING_MINT_SIZE
    }
}

pub struct DataContractState<'a> {
    data: &'a mut [u8],
}

impl<'a> DataContractState<'a> {
    pub fn new(data: &'a mut [u8]) -> Self {
        Self { data }
    }

    pub fn get_header(&self) -> &DataContractStateHeader {
        bytemuck::from_bytes(&self.data[0..HEADER_SIZE])
    }

    pub fn get_header_mut(&mut self) -> &mut DataContractStateHeader {
        bytemuck::from_bytes_mut(&mut self.data[0..HEADER_SIZE])
    }

    pub fn calculate_target_size(pending_mints_count: u16) -> usize {
        let groups = (pending_mints_count as usize + MAX_PENDING_MINTS_PER_GROUP - 1)
            / MAX_PENDING_MINTS_PER_GROUP;

        HEADER_SIZE + (groups * 32) + (pending_mints_count as usize * PENDING_MINT_SIZE)
    }

    pub fn get_nth_group_hash_offset(group_idx: u16) -> usize {
        HEADER_SIZE + (group_idx as usize * 32)
    }

    pub fn get_nth_pending_mint_offset(&self, global_mint_idx: u16) -> usize {
        let h = self.get_header();
        HEADER_SIZE
            + (h.pending_mint_groups_count as usize * 32)
            + (global_mint_idx as usize * PENDING_MINT_SIZE)
    }

    pub fn insert_pending_mints(&mut self, group_index: u16, mint_data: &[u8]) -> ProgramResult {
        let header = self.get_header();

        if header.is_locked != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        if group_index >= header.pending_mint_groups_count {
            return Err(ProgramError::InvalidArgument);
        }

        if mint_data.len() > MAX_PENDING_MINTS_PER_GROUP * PENDING_MINT_SIZE {
            return Err(ProgramError::InvalidInstructionData);
        }
        if mint_data.len() % PENDING_MINT_SIZE != 0 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let global_start = group_index * MAX_PENDING_MINTS_PER_GROUP_U16;
        let hash_offset = Self::get_nth_group_hash_offset(group_index);
        let mint_offset = self.get_nth_pending_mint_offset(global_start);

        let data_end = mint_offset + mint_data.len();
        let hash_end = hash_offset + 32;

        if data_end > self.data.len() || hash_end > self.data.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        let existing_hash = &self.data[hash_offset..hash_end];
        if existing_hash != &[0u8; 32] {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let digest = hash_impl_sha256_bytes(mint_data);
        self.data[hash_offset..hash_end].copy_from_slice(&digest);
        self.data[mint_offset..data_end].copy_from_slice(mint_data);

        let count_inc = (mint_data.len() / PENDING_MINT_SIZE) as u16;
        self.get_header_mut().pending_mints_initialized += count_inc;

        Ok(())
    }
}
#[cfg(feature = "solprogram")]
entrypoint!(process_instruction);

#[cfg(feature = "solprogram")]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }
    let (tag, rest) = instruction_data.split_first().unwrap();

    let account_info_iter = &mut accounts.iter();
    let storage_account = next_account_info(account_info_iter)?;

    if storage_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    let require_signer = |acc: &AccountInfo| -> ProgramResult {
        if !acc.is_signer {
            Err(ProgramError::MissingRequiredSignature)
        } else {
            Ok(())
        }
    };

    match tag {
        // 0: Setup(auth_key: [u8;32])
        0 => {
            if rest.len() != 32 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let auth_key: [u8; 32] = rest[0..32].try_into().unwrap();

            if storage_account.data_len() < HEADER_SIZE {
                return Err(ProgramError::UninitializedAccount);
            }

            let mut data = storage_account.try_borrow_mut_data()?;
            data[0..HEADER_SIZE].fill(0); // Zero Header
            DataContractState::new(&mut data)
                .get_header_mut()
                .setup(auth_key)?;
            msg!("Setup Complete");
        }

        // 1: Reinit(total_mints: u16)
        1 => {
            let payer = next_account_info(account_info_iter)?;
            let system_program_acc = next_account_info(account_info_iter)?;
            require_signer(payer)?;

            if rest.len() != 2 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let count = u16::from_le_bytes(rest[0..2].try_into().unwrap());

            let target_size = DataContractState::calculate_target_size(count);
            let current_len = storage_account.data_len();
            let mut new_len = current_len;

            if target_size > current_len {
                let increase = (target_size - current_len).min(MAX_PERMITTED_DATA_INCREASE);
                new_len = current_len + increase;
            } else if target_size < current_len {
                new_len = target_size;
            }

            if new_len != current_len {
                realloc_account(storage_account, payer, system_program_acc, new_len, true)?;
            }

            let mut data = storage_account.try_borrow_mut_data()?;
            if data.len() < HEADER_SIZE {
                return Err(ProgramError::UninitializedAccount);
            }

            // 1. Update Header counts
            let mut wrapper = DataContractState::new(&mut data);
            wrapper.get_header_mut().reinit(count)?;

            // 2. CRITICAL FIX: Zero out the Hashes area
            // We need to clear the hash slots so Insert doesn't think data is already there.
            // Hash area starts at HEADER_SIZE.
            // Length = groups * 32.
            let groups = wrapper.get_header().pending_mint_groups_count as usize;
            let hash_area_size = groups * 32;
            let hash_start = HEADER_SIZE;
            let hash_end = hash_start + hash_area_size;

            // Ensure we don't go out of bounds (realloc might have limited us)
            let actual_len = wrapper.data.len();
            if actual_len >= hash_end {
                wrapper.data[hash_start..hash_end].fill(0);
            } else if actual_len > hash_start {
                // Should technically not happen if realloc worked, but safe fallback
                wrapper.data[hash_start..actual_len].fill(0);
            }

            msg!("Reinit Complete: {}", count);
        }

        // 2: Resize()
        2 => {
            let payer = next_account_info(account_info_iter)?;
            let system_program_acc = next_account_info(account_info_iter)?;
            require_signer(payer)?;

            let (target_size, is_locked) = {
                let data = storage_account.try_borrow_data()?;
                if data.len() < HEADER_SIZE {
                    return Err(ProgramError::UninitializedAccount);
                }
                let h = bytemuck::from_bytes::<DataContractStateHeader>(&data[0..HEADER_SIZE]);
                (h.get_total_buffer_size(), h.is_locked)
            };

            if is_locked != 0 {
                return Err(ProgramError::AccountAlreadyInitialized);
            }

            let current_len = storage_account.data_len();

            if target_size < current_len {
                msg!("Resize Error: Target size smaller than current. Use Reinit.");
                return Err(ProgramError::InvalidAccountData);
            }

            if target_size > current_len {
                let increase = (target_size - current_len).min(MAX_PERMITTED_DATA_INCREASE);
                let new_len = current_len + increase;
                realloc_account(storage_account, payer, system_program_acc, new_len, true)?;
                msg!("Expanded: {} -> {}", current_len, new_len);
            }
        }

        // 3: Insert
        3 => {
            let payer = next_account_info(account_info_iter)?;
            let system_program_acc = next_account_info(account_info_iter)?;
            require_signer(payer)?;

            if rest.len() < 2 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let (idx_bytes, mint_data) = rest.split_at(2);
            let group_index = u16::from_le_bytes(idx_bytes.try_into().unwrap());

            let target_size = {
                let data = storage_account.try_borrow_data()?;
                if data.len() < HEADER_SIZE {
                    return Err(ProgramError::UninitializedAccount);
                }
                let h = bytemuck::from_bytes::<DataContractStateHeader>(&data[0..HEADER_SIZE]);
                h.get_total_buffer_size()
            };

            let current_len = storage_account.data_len();
            if current_len < target_size {
                let increase = (target_size - current_len).min(MAX_PERMITTED_DATA_INCREASE);
                let new_len = current_len + increase;
                realloc_account(storage_account, payer, system_program_acc, new_len, false)?;
            }

            let mut data = storage_account.try_borrow_mut_data()?;
            DataContractState::new(&mut data).insert_pending_mints(group_index, mint_data)?;
            msg!("Insert Group {} Success", group_index);
        }

        // 4: Lock
        4 => {
            let signer = next_account_info(account_info_iter)?;
            require_signer(signer)?;
            let mut data = storage_account.try_borrow_mut_data()?;
            DataContractState::new(&mut data)
                .get_header_mut()
                .lock(signer.key.to_bytes())?;
            msg!("Locked");
        }

        // 5: Unlock
        5 => {
            let signer = next_account_info(account_info_iter)?;
            require_signer(signer)?;
            let mut data = storage_account.try_borrow_mut_data()?;
            DataContractState::new(&mut data)
                .get_header_mut()
                .unlock(signer.key.to_bytes())?;
            msg!("Unlocked");
        }

        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}
