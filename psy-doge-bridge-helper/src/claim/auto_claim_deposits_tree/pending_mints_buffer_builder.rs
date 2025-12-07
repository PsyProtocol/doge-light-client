use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_bytes};


const MAX_PENDING_MINTS_PER_GROUP: usize = 24;
/*
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct PendingMint {
    pub recipient: [u8; 32],
    pub amount: u64,
}
*/
const PENDING_MINT_SIZE: usize = 40;

pub struct PendingMintsGroupsBuilder {
    pub group_hashes_buffer: Vec<u8>,
    pub current_group: Vec<u8>,
    pub next_item_in_group_index: usize,
    pub total_groups: usize,
}

impl PendingMintsGroupsBuilder {
    pub fn new() -> Self {
        Self {
            group_hashes_buffer: vec![0u8; 2],
            current_group: vec![0u8; MAX_PENDING_MINTS_PER_GROUP * PENDING_MINT_SIZE],
            next_item_in_group_index: 0,
            total_groups: 0,
        }
    }
    pub fn new_with_hint(total_items_hint: usize) -> Self {
        let mut total_groups_hint = total_items_hint / MAX_PENDING_MINTS_PER_GROUP;
        if (total_groups_hint * MAX_PENDING_MINTS_PER_GROUP) < total_items_hint {
            total_groups_hint += 1;
        }
        let mut group_hashes_buffer = Vec::with_capacity(2 + total_groups_hint * 32);
        group_hashes_buffer.extend_from_slice(&[0u8; 2]);


        Self {
            group_hashes_buffer: group_hashes_buffer,
            current_group: vec![0u8; MAX_PENDING_MINTS_PER_GROUP * PENDING_MINT_SIZE],
            next_item_in_group_index: 0,
            total_groups: 0,
        }
    }

    pub fn append_pending_mint(&mut self, recipient_solana_public_key: &[u8; 32], amount: u64)  {

        let amount_bytes = amount.to_le_bytes();
        if self.next_item_in_group_index == MAX_PENDING_MINTS_PER_GROUP {
            let hash = hash_impl_sha256_bytes(&self.current_group[..MAX_PENDING_MINTS_PER_GROUP * PENDING_MINT_SIZE]);
            self.group_hashes_buffer.extend_from_slice(&hash);
            self.next_item_in_group_index = 0;
            self.total_groups += 1;
        }


        self.current_group[self.next_item_in_group_index * PENDING_MINT_SIZE..self.next_item_in_group_index * PENDING_MINT_SIZE + 32]
            .copy_from_slice(recipient_solana_public_key);
        self.current_group[self.next_item_in_group_index * PENDING_MINT_SIZE + 32..self.next_item_in_group_index * PENDING_MINT_SIZE + 40]
            .copy_from_slice(&amount_bytes);
        self.next_item_in_group_index += 1;
    }

    pub fn finalize(mut self) -> anyhow::Result<QHash256> {
        if self.total_groups != 0 || self.next_item_in_group_index > 0 {
            let total_items = self.total_groups * MAX_PENDING_MINTS_PER_GROUP + self.next_item_in_group_index;
            if self.next_item_in_group_index > 0 {
                let hash = hash_impl_sha256_bytes(&self.current_group[..self.next_item_in_group_index * PENDING_MINT_SIZE]);
                self.group_hashes_buffer.extend_from_slice(&hash);
                self.total_groups += 1;
            }
            if total_items > u16::MAX as usize {
                return Err(anyhow::anyhow!("Too many pending mints: {}", total_items));
            }
            let total_items_u16 = total_items as u16;
            self.group_hashes_buffer[0..2].copy_from_slice(&total_items_u16.to_le_bytes());
        }
        Ok(hash_impl_sha256_bytes(&self.group_hashes_buffer))
    }
}