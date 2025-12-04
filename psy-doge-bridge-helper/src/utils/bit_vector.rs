use doge_light_client::common_types::QHash256;



#[inline(always)]
pub const fn get_bit_in_bit_vector(bit_vector: &QHash256, bit_index: u8) -> bool {
    bit_vector[ (bit_index >> 3) as usize ] & (1 << (bit_index & 0x07)) != 0
}

#[inline]
pub fn set_bit_in_bit_vector(bit_vector: &mut QHash256, bit_index: u8) {
    bit_vector[ (bit_index >> 3) as usize ] |= 1 << (bit_index & 0x07);
}

pub fn set_bit_in_bit_vector_cloned(bit_vector: &QHash256, bit_index: u8) -> QHash256 {
    let mut new_bit_vector = *bit_vector;
    set_bit_in_bit_vector(&mut new_bit_vector, bit_index);
    new_bit_vector
}


