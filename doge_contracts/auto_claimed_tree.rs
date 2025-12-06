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

use bytemuck::{Pod, Zeroable};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    hash::hash,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

// ============================================================================
// Constants & Structs
// ============================================================================

const MAX_PENDING_MINTS_PER_GROUP: usize = 24;
const MAX_PENDING_MINTS_PER_GROUP_U16: u16 = MAX_PENDING_MINTS_PER_GROUP as u16;
const MAX_PERMITTED_DATA_INCREASE: usize = 10_240;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct PendingMint {
    pub recipient: [u8; 32],
    pub amount: u64,
}
const PENDING_MINT_SIZE: usize = std::mem::size_of::<PendingMint>();

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
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

// ============================================================================
// Helpers
// ============================================================================

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

        let digest = hash(mint_data).to_bytes();
        self.data[hash_offset..hash_end].copy_from_slice(&digest);
        self.data[mint_offset..data_end].copy_from_slice(mint_data);

        let count_inc = (mint_data.len() / PENDING_MINT_SIZE) as u16;
        self.get_header_mut().pending_mints_initialized += count_inc;

        Ok(())
    }
}

entrypoint!(process_instruction);

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


/*
const MAX_PENDING_MINTS_PER_GROUP = 24;
const PENDING_MINT_SIZE = 40;
const HEADER_SIZE = 40;
const HASH_SIZE = 32;

const IX_SETUP = 0;
const IX_REINIT = 1;
const IX_RESIZE = 2;
const IX_INSERT = 3;
const IX_LOCK = 4;
const IX_UNLOCK = 5;

function u8ArrayToHex(buffer) {
  return Array.from(buffer)
    .slice(0, 100)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function hexAndHeader(buffer) {
  const isLocked = buffer.readUInt8(32);
  const groups = buffer.readUInt16LE(34);
  const init = buffer.readUInt16LE(36);
  const count = buffer.readUInt16LE(38);
  const hex = Array.from(buffer)
    .slice(0, 100)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");

  return `
    isLocked: ${isLocked},
    groups: ${groups},
    init: ${init},
    count: ${count},
    hex[0..100]: ${hex},
  `;
}

function printHeaderRaw(buffer) {
  if (buffer.length < HEADER_SIZE) return;
  console.log("header: " + u8ArrayToHex(buffer));
  const isLocked = buffer.readUInt8(32);
  const groups = buffer.readUInt16LE(34);
  const init = buffer.readUInt16LE(36);
  const count = buffer.readUInt16LE(38);
  console.log(
    `[State] Locked: ${isLocked}, Groups: ${groups}, Init: ${init}, Count: ${count}`
  );
}

function calculateTargetSize(pendingMintsCount) {
  const groups = Math.ceil(pendingMintsCount / MAX_PENDING_MINTS_PER_GROUP);
  return (
    HEADER_SIZE + groups * HASH_SIZE + pendingMintsCount * PENDING_MINT_SIZE
  );
}

function decodeHeader(buffer) {
  if (buffer.length < HEADER_SIZE) return null;
  return {
    authorizedLocker: new web3.PublicKey(buffer.subarray(0, 32)),
    isLocked: buffer.readUInt8(32),
    mode: buffer.readUInt8(33),
    pendingMintGroupsCount: buffer.readUInt16LE(34),
    pendingMintsInitialized: buffer.readUInt16LE(36),
    pendingMintsCount: buffer.readUInt16LE(38),
  };
}

function createMintData(recipientPubkey, amount) {
  const buf = Buffer.alloc(40);
  buf.set(recipientPubkey.toBuffer(), 0);
  buf.writeBigUInt64LE(BigInt(amount), 32);
  return buf;
}

describe("Storage Contract Tests (5KB)", function () {
  this.timeout(90000); // 90 seconds timeout

  let storageKp;
  let authorityKp;

  before(async () => {
    storageKp = new web3.Keypair();
    authorityKp = pg.wallet.keypair;
    console.log("Storage Account:", storageKp.publicKey.toString());
  });

  it("1. Setup", async () => {
    const lamports = await pg.connection.getMinimumBalanceForRentExemption(
      HEADER_SIZE
    );

    const createAccountIx = web3.SystemProgram.createAccount({
      fromPubkey: pg.wallet.publicKey,
      newAccountPubkey: storageKp.publicKey,
      lamports: lamports,
      space: HEADER_SIZE,
      programId: pg.PROGRAM_ID,
    });

    const setupData = Buffer.alloc(1 + 32);
    setupData.writeUInt8(IX_SETUP, 0);
    setupData.set(authorityKp.publicKey.toBuffer(), 1);

    const setupIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
      ],
      programId: pg.PROGRAM_ID,
      data: setupData,
    });

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(createAccountIx, setupIx),
      [pg.wallet.keypair, storageKp]
    );

    const acc = await pg.connection.getAccountInfo(storageKp.publicKey);
    printHeaderRaw(acc.data);
    const header = decodeHeader(acc.data);
    assert.equal(header.isLocked, 0);
    assert(header.authorizedLocker.equals(authorityKp.publicKey));
  });

  it("2. Expansion Test (125 Mints ~5.2KB)", async () => {
    const TOTAL_MINTS = 125;
    const TARGET_SIZE = calculateTargetSize(TOTAL_MINTS);
    console.log(
      `Targeting ${TOTAL_MINTS} mints. Required Size: ${TARGET_SIZE}`
    );

    const reinitData = Buffer.alloc(3);
    reinitData.writeUInt8(IX_REINIT, 0);
    reinitData.writeUInt16LE(TOTAL_MINTS, 1);

    const reinitIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: reinitData,
    });

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(reinitIx),
      [pg.wallet.keypair]
    );

    let acc = await pg.connection.getAccountInfo(storageKp.publicKey);
    printHeaderRaw(acc.data);
    assert.equal(acc.data.length, TARGET_SIZE);
  });

  it("3. Insert Batches", async () => {
    const groupsToInsert = [0, 1, 5];
    const recipient = new web3.Keypair().publicKey;

    for (const groupIdx of groupsToInsert) {
      const isLast = groupIdx === 5;
      const countInGroup = isLast ? 125 % 24 : 24;
      const countToUse = countInGroup === 0 ? 24 : countInGroup;

      const mintsBuffer = Buffer.alloc(countToUse * PENDING_MINT_SIZE);
      for (let i = 0; i < countToUse; i++) {
        const m = createMintData(recipient, 100 + i);
        m.copy(mintsBuffer, i * PENDING_MINT_SIZE);
      }

      const ixData = Buffer.alloc(1 + 2 + mintsBuffer.length);
      ixData.writeUInt8(IX_INSERT, 0);
      ixData.writeUInt16LE(groupIdx, 1);
      mintsBuffer.copy(ixData, 3);

      const insertIx = new web3.TransactionInstruction({
        keys: [
          { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
          { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
          {
            pubkey: web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
          },
        ],
        programId: pg.PROGRAM_ID,
        data: ixData,
      });

      await web3.sendAndConfirmTransaction(
        pg.connection,
        new web3.Transaction().add(insertIx),
        [pg.wallet.keypair]
      );
      console.log(`Inserted Group ${groupIdx}`);
    }

    const acc = await pg.connection.getAccountInfo(storageKp.publicKey);
    printHeaderRaw(acc.data);
    const header = decodeHeader(acc.data);
    // 24 + 24 + 5 = 53
    assert.equal(header.pendingMintsInitialized, 53);
  });

  it("4. Lock (Shrink & Lock)", async () => {
    // 1. Reinit to 53 (Shrink)
    const insertedCount = 53;
    const shrinkData = Buffer.alloc(3);
    shrinkData.writeUInt8(IX_REINIT, 0);
    shrinkData.writeUInt16LE(insertedCount, 1);

    console.log(
      "4.1 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );
    const shrinkIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: shrinkData,
    });
    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(shrinkIx),
      [pg.wallet.keypair]
    );
    console.log("shrink compelted");
    console.log(
      "4.2 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    // 2. Reinit to 24 (Group 0) for quick lock test
    shrinkData.writeUInt16LE(24, 1);
    const shrinkIx2 = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: shrinkData,
    });
    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(shrinkIx2),
      [pg.wallet.keypair]
    );
    console.log(
      "4.3 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    // 3. Re-Insert Group 0 (24 items)
    const recipient = new web3.Keypair().publicKey;
    const mintsBuffer = Buffer.alloc(24 * PENDING_MINT_SIZE);
    for (let i = 0; i < 24; i++) {
      createMintData(recipient, 500 + i).copy(
        mintsBuffer,
        i * PENDING_MINT_SIZE
      );
    }
    const insertData = Buffer.alloc(1 + 2 + mintsBuffer.length);
    insertData.writeUInt8(IX_INSERT, 0);
    insertData.writeUInt16LE(0, 1);
    mintsBuffer.copy(insertData, 3);

    const insertIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: insertData,
    });
    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(insertIx),
      [pg.wallet.keypair]
    );
    console.log(
      "4.4 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    // 4. Lock
    const lockData = Buffer.alloc(1);
    lockData.writeUInt8(IX_LOCK, 0);
    const lockIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: authorityKp.publicKey, isSigner: true, isWritable: false },
      ],
      programId: pg.PROGRAM_ID,
      data: lockData,
    });

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(lockIx),
      [pg.wallet.keypair]
    );
    console.log(
      "4.5 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    let acc = await pg.connection.getAccountInfo(storageKp.publicKey);
    printHeaderRaw(acc.data);
    const header = decodeHeader(acc.data);
    assert.equal(header.isLocked, 1);
  });

  it("5. Security: Fail to Resize when Locked", async () => {
    const resizeData = Buffer.alloc(1);
    resizeData.writeUInt8(IX_RESIZE, 0);

    const resizeIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: resizeData,
    });

    try {
      await web3.sendAndConfirmTransaction(
        pg.connection,
        new web3.Transaction().add(resizeIx),
        [pg.wallet.keypair]
      );
      assert.fail("Should have thrown error");
    } catch (err) {
      console.log("Correctly failed to resize while locked.");
    }
  });

  it("6. Unlock and Shrink to Minimum", async () => {
    // 1. Unlock
    const unlockData = Buffer.alloc(1);
    unlockData.writeUInt8(IX_UNLOCK, 0);
    console.log(
      "6.1 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    const unlockIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: authorityKp.publicKey, isSigner: true, isWritable: false },
      ],
      programId: pg.PROGRAM_ID,
      data: unlockData,
    });
    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(unlockIx),
      [pg.wallet.keypair]
    );
    console.log(
      "6.2 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    // 2. Shrink to 0
    const reinitData = Buffer.alloc(3);
    reinitData.writeUInt8(IX_REINIT, 0);
    reinitData.writeUInt16LE(0, 1);

    const shrinkIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: storageKp.publicKey, isSigner: false, isWritable: true },
        { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: pg.PROGRAM_ID,
      data: reinitData,
    });

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(shrinkIx),
      [pg.wallet.keypair]
    );
    console.log(
      "6.3 header: " +
        hexAndHeader(
          (await pg.connection.getAccountInfo(storageKp.publicKey)).data
        )
    );

    const acc = await pg.connection.getAccountInfo(storageKp.publicKey);
    assert.equal(acc.data.length, HEADER_SIZE);
    console.log("Account completely reset and shrunk.");
  });
});


*/