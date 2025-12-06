use bytemuck::{Pod, Zeroable};
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

// ============================================================================
// Constants & Structs
// ============================================================================

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct SimpleDataContractHeader {
    // Offset 0
    pub authorized_writer: [u8; 32],
    // Offset 32
    pub init_status: u16,
    // Offset 34
    pub finalized_status: u16,
    // Offset 36
    pub doge_block_height: u32,
    // Offset 40
    pub batch_id: u32,
    // Offset 44
    pub data_size: u32,
    // Total Size: 48 bytes
}

const HEADER_SIZE: usize = std::mem::size_of::<SimpleDataContractHeader>();
const _ASSERT_SIZE: () = assert!(HEADER_SIZE == 48);

// ============================================================================
// Helpers
// ============================================================================

pub fn realloc_account<'a>(
    account: &AccountInfo<'a>,
    payer: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    new_size: usize,
) -> ProgramResult {
    let rent = Rent::get()?;
    let current_len = account.data_len();
    let old_rent = rent.minimum_balance(current_len);
    let new_rent = rent.minimum_balance(new_size);

    if new_rent > old_rent {
        let diff = new_rent - old_rent;
        invoke(
            &system_instruction::transfer(payer.key, account.key, diff),
            &[payer.clone(), account.clone(), system_program.clone()],
        )?;
    } else if old_rent > new_rent {
        let diff = old_rent - new_rent;
        let mut from_lamports = account.try_borrow_mut_lamports()?;
        let mut to_lamports = payer.try_borrow_mut_lamports()?;
        **from_lamports -= diff;
        **to_lamports += diff;
    }

    account.realloc(new_size, false)?;
    Ok(())
}

fn handle_batch_transition(
    header: &mut SimpleDataContractHeader,
    input_batch_id: u32,
) -> ProgramResult {
    if input_batch_id == header.batch_id {
        if header.finalized_status == 1 {
            msg!("Error: Batch {} is finalized", input_batch_id);
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    } else if input_batch_id == header.batch_id + 1 {
        header.batch_id = input_batch_id;
        header.finalized_status = 0;
    } else {
        msg!(
            "Error: Invalid Batch ID. Current: {}, Input: {}",
            header.batch_id,
            input_batch_id
        );
        return Err(ProgramError::InvalidArgument);
    }
    Ok(())
}

// ============================================================================
// Entrypoint
// ============================================================================

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
    let contract_account = next_account_info(account_info_iter)?;

    if contract_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    match tag {
        // --------------------------------------------------------------------
        // 0: Initialize(authorized_writer: [u8; 32])
        // --------------------------------------------------------------------
        0 => {
            if rest.len() != 32 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let auth_key: [u8; 32] = rest[0..32].try_into().unwrap();

            if contract_account.data_len() < HEADER_SIZE {
                return Err(ProgramError::UninitializedAccount);
            }

            let mut data = contract_account.try_borrow_mut_data()?;
            data[0..HEADER_SIZE].fill(0);

            let header =
                bytemuck::from_bytes_mut::<SimpleDataContractHeader>(&mut data[0..HEADER_SIZE]);

            header.authorized_writer = auth_key;
            header.init_status = 1;

            msg!("Initialized. Writer set.");
        }

        // --------------------------------------------------------------------
        // 1: SetDataLength(new_len: u32, resize: bool, batch_id: u32, height: u32, finalize: bool)
        // --------------------------------------------------------------------
        1 => {
            if rest.len() != 14 {
                return Err(ProgramError::InvalidInstructionData);
            }

            let new_length = u32::from_le_bytes(rest[0..4].try_into().unwrap());
            let resize = rest[4] != 0;
            let input_batch_id = u32::from_le_bytes(rest[5..9].try_into().unwrap());
            let input_doge_height = u32::from_le_bytes(rest[9..13].try_into().unwrap());
            let finalize = rest[13] != 0;

            let payer = next_account_info(account_info_iter)?;
            let system_program = next_account_info(account_info_iter)?;
            let signer = next_account_info(account_info_iter)?;

            if !signer.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            // Scope 1: Header Logic
            {
                let mut data = contract_account.try_borrow_mut_data()?;
                let header =
                    bytemuck::from_bytes_mut::<SimpleDataContractHeader>(&mut data[0..HEADER_SIZE]);

                if header.authorized_writer != signer.key.to_bytes() {
                    return Err(ProgramError::IllegalOwner);
                }

                handle_batch_transition(header, input_batch_id)?;

                // Doge Block Height Logic
                if finalize {
                    if header.doge_block_height != 0 {
                        let lower = header.doge_block_height.saturating_sub(1000);
                        let upper = header.doge_block_height.saturating_add(1000);

                        if input_doge_height < lower || input_doge_height > upper {
                            msg!(
                                "Doge Height Error. Stored: {}, Input: {}",
                                header.doge_block_height,
                                input_doge_height
                            );
                            return Err(ProgramError::InvalidInstructionData);
                        }
                    }
                } else {
                    if input_doge_height != header.doge_block_height {
                        msg!(
                            "Doge Height mismatch. Stored: {}, Input: {}",
                            header.doge_block_height,
                            input_doge_height
                        );
                        return Err(ProgramError::InvalidInstructionData);
                    }
                }

                header.doge_block_height = input_doge_height;

                if finalize {
                    header.finalized_status = 1;
                }
            } // Drop borrow

            // Scope 2: Realloc
            if resize {
                let required_physical = HEADER_SIZE + (new_length as usize);
                realloc_account(contract_account, payer, system_program, required_physical)?;
            } else {
                if contract_account.data_len() < HEADER_SIZE + new_length as usize {
                    return Err(ProgramError::AccountDataTooSmall);
                }
            }

            // Scope 3: Update Size
            {
                let mut data = contract_account.try_borrow_mut_data()?;
                let header =
                    bytemuck::from_bytes_mut::<SimpleDataContractHeader>(&mut data[0..HEADER_SIZE]);
                header.data_size = new_length;
                msg!("SetDataLength Success. Batch: {}", header.batch_id);
            }
        }

        // --------------------------------------------------------------------
        // 2: WriteData(batch_id: u32, offset: u32, bytes: [u8...])
        // --------------------------------------------------------------------
        2 => {
            if rest.len() < 8 {
                return Err(ProgramError::InvalidInstructionData);
            }
            let input_batch_id = u32::from_le_bytes(rest[0..4].try_into().unwrap());
            let offset = u32::from_le_bytes(rest[4..8].try_into().unwrap());
            let raw_data = &rest[8..];

            let signer = next_account_info(account_info_iter)?;
            if !signer.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }

            let mut data = contract_account.try_borrow_mut_data()?;

            // SECURITY FIX: Split slice to allow concurrent mutable borrow
            let (header_bytes, body_bytes) = data.split_at_mut(HEADER_SIZE);
            let header = bytemuck::from_bytes_mut::<SimpleDataContractHeader>(header_bytes);

            if header.authorized_writer != signer.key.to_bytes() {
                return Err(ProgramError::IllegalOwner);
            }

            handle_batch_transition(header, input_batch_id)?;

            let write_end_logical = offset as usize + raw_data.len();

            // Check against actual allocated body size
            if write_end_logical > body_bytes.len() {
                msg!(
                    "Write OOB. BodyLen: {}, Req: {}",
                    body_bytes.len(),
                    write_end_logical
                );
                return Err(ProgramError::AccountDataTooSmall);
            }

            // Write to body slice (offset is relative to start of body)
            let dest = &mut body_bytes[offset as usize..write_end_logical];
            dest.copy_from_slice(raw_data);

            if write_end_logical > header.data_size as usize {
                header.data_size = write_end_logical as u32;
            }

            msg!("WriteData Success.");
        }

        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}


/*
const HEADER_SIZE = 48;

// Instructions
const IX_INIT = 0;
const IX_SET_LEN = 1;
const IX_WRITE = 2;

function u8ArrayToHex(buffer) {
  return Array.from(buffer)
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

function decodeHeader(buffer) {
  if (buffer.length < HEADER_SIZE) return null;
  // [Auth:32][Init:2][Final:2][Doge:4][Batch:4][Size:4]
  return {
    authorizedWriter: new web3.PublicKey(buffer.subarray(0, 32)),
    initStatus: buffer.readUInt16LE(32),
    finalizedStatus: buffer.readUInt16LE(34),
    dogeBlockHeight: buffer.readUInt32LE(36),
    batchId: buffer.readUInt32LE(40),
    dataSize: buffer.readUInt32LE(44),
  };
}

describe("Simple Data Contract (Batching + Height Logic)", function () {
  this.timeout(60000);

  let contractKp;
  let writerKp;

  before(async () => {
    contractKp = new web3.Keypair();
    writerKp = pg.wallet.keypair;
    console.log("Contract:", contractKp.publicKey.toString());
  });

  it("1. Initialize", async () => {
    const lamports = await pg.connection.getMinimumBalanceForRentExemption(
      HEADER_SIZE
    );

    const createIx = web3.SystemProgram.createAccount({
      fromPubkey: pg.wallet.publicKey,
      newAccountPubkey: contractKp.publicKey,
      lamports,
      space: HEADER_SIZE,
      programId: pg.PROGRAM_ID,
    });

    const initData = Buffer.alloc(1 + 32);
    initData.writeUInt8(IX_INIT, 0);
    initData.set(writerKp.publicKey.toBuffer(), 1);

    const initIx = new web3.TransactionInstruction({
      keys: [
        { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      ],
      programId: pg.PROGRAM_ID,
      data: initData,
    });

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(createIx, initIx),
      [pg.wallet.keypair, contractKp]
    );

    const acc = await pg.connection.getAccountInfo(contractKp.publicKey);
    const header = decodeHeader(acc.data);

    assert.equal(header.initStatus, 1);
    assert.equal(header.batchId, 0);
    assert.equal(header.dogeBlockHeight, 0);
    assert.equal(header.finalizedStatus, 0);
  });

  it("2. Set Data Length (Batch 0, Height 0)", async () => {
    const NEW_SIZE = 64;
    const BATCH_ID = 0;
    const DOGE_HEIGHT = 0;
    const FINALIZE = 0;

    const data = Buffer.alloc(15);
    let offset = 0;
    data.writeUInt8(IX_SET_LEN, offset++);
    data.writeUInt32LE(NEW_SIZE, offset);
    offset += 4;
    data.writeUInt8(1, offset++);
    data.writeUInt32LE(BATCH_ID, offset);
    offset += 4;
    data.writeUInt32LE(DOGE_HEIGHT, offset);
    offset += 4;
    data.writeUInt8(FINALIZE, offset++);

    const keys = [
      { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: writerKp.publicKey, isSigner: true, isWritable: false },
    ];

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(
        new web3.TransactionInstruction({
          keys,
          programId: pg.PROGRAM_ID,
          data,
        })
      ),
      [pg.wallet.keypair]
    );

    const acc = await pg.connection.getAccountInfo(contractKp.publicKey);
    const header = decodeHeader(acc.data);
    assert.equal(header.dataSize, NEW_SIZE);
  });

  it("3. Write Data (Batch 0)", async () => {
    const BATCH_ID = 0;
    const OFFSET = 0;
    const PAYLOAD = Buffer.from([0xaa, 0xbb, 0xcc]);

    const data = Buffer.alloc(9 + PAYLOAD.length);
    data.writeUInt8(IX_WRITE, 0);
    data.writeUInt32LE(BATCH_ID, 1);
    data.writeUInt32LE(OFFSET, 5);
    PAYLOAD.copy(data, 9);

    const keys = [
      { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      { pubkey: writerKp.publicKey, isSigner: true, isWritable: false },
    ];

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(
        new web3.TransactionInstruction({
          keys,
          programId: pg.PROGRAM_ID,
          data,
        })
      ),
      [pg.wallet.keypair]
    );
  });

  it("4. Finalize Batch 0 (Initial Jump Test)", async () => {
    const NEW_SIZE = 64;
    const BATCH_ID = 0;
    const DOGE_HEIGHT = 5000;
    const FINALIZE = 1; // true

    const data = Buffer.alloc(15);
    let offset = 0;
    data.writeUInt8(IX_SET_LEN, offset++);
    data.writeUInt32LE(NEW_SIZE, offset);
    offset += 4;
    data.writeUInt8(0, offset++);
    data.writeUInt32LE(BATCH_ID, offset);
    offset += 4;
    data.writeUInt32LE(DOGE_HEIGHT, offset);
    offset += 4;
    data.writeUInt8(FINALIZE, offset++);

    const keys = [
      { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: writerKp.publicKey, isSigner: true, isWritable: false },
    ];

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(
        new web3.TransactionInstruction({
          keys,
          programId: pg.PROGRAM_ID,
          data,
        })
      ),
      [pg.wallet.keypair]
    );

    const acc = await pg.connection.getAccountInfo(contractKp.publicKey);
    const header = decodeHeader(acc.data);
    assert.equal(header.finalizedStatus, 1);
    assert.equal(header.dogeBlockHeight, 5000);
    console.log("Successfully jumped from 0 to 5000.");
  });

  it("5. Start Batch 1 (Implicit Unfinalize)", async () => {
    const BATCH_ID = 1;
    const OFFSET = 0;
    const PAYLOAD = Buffer.from([0xff]);

    const data = Buffer.alloc(9 + PAYLOAD.length);
    data.writeUInt8(IX_WRITE, 0);
    data.writeUInt32LE(BATCH_ID, 1);
    data.writeUInt32LE(OFFSET, 5);
    PAYLOAD.copy(data, 9);

    const keys = [
      { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      { pubkey: writerKp.publicKey, isSigner: true, isWritable: false },
    ];

    await web3.sendAndConfirmTransaction(
      pg.connection,
      new web3.Transaction().add(
        new web3.TransactionInstruction({
          keys,
          programId: pg.PROGRAM_ID,
          data,
        })
      ),
      [pg.wallet.keypair]
    );

    const acc = await pg.connection.getAccountInfo(contractKp.publicKey);
    const header = decodeHeader(acc.data);
    assert.equal(header.batchId, 1);
    assert.equal(header.finalizedStatus, 0);
  });

  it("6. Fail: Doge Height Logic (Too Big Jump)", async () => {
    const BATCH_ID = 1;
    const DOGE_HEIGHT = 7000;
    const FINALIZE = 1;

    const data = Buffer.alloc(15);
    let offset = 0;
    data.writeUInt8(IX_SET_LEN, offset++);
    data.writeUInt32LE(64, offset);
    offset += 4;
    data.writeUInt8(0, offset++);
    data.writeUInt32LE(BATCH_ID, offset);
    offset += 4;
    data.writeUInt32LE(DOGE_HEIGHT, offset);
    offset += 4;
    data.writeUInt8(FINALIZE, offset++);

    const keys = [
      { pubkey: contractKp.publicKey, isSigner: false, isWritable: true },
      { pubkey: pg.wallet.publicKey, isSigner: true, isWritable: true },
      {
        pubkey: web3.SystemProgram.programId,
        isSigner: false,
        isWritable: false,
      },
      { pubkey: writerKp.publicKey, isSigner: true, isWritable: false },
    ];

    try {
      await web3.sendAndConfirmTransaction(
        pg.connection,
        new web3.Transaction().add(
          new web3.TransactionInstruction({
            keys,
            programId: pg.PROGRAM_ID,
            data,
          })
        ),
        [pg.wallet.keypair]
      );
      assert.fail("Should fail (Jump too large from non-zero)");
    } catch (e) {
      console.log("Correctly rejected large jump from 5000 to 7000.");
    }
  });
});


*/