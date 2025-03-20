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

use crate::core_data::QHash256;

const DIFFICULTY_NEGATIVE_FLAG: u32 = 0x00800000;
fn find_first_non_zero_index(x: [u8; 32]) -> i32 {
    for i in 0..32 {
        if x[i] != 0 {
            return i as i32;
        }
    }
    -1
}
fn num_bits_u256_be_and_first_non_zero_index(x: [u8; 32]) -> (usize, usize) {
    let first_non_zero_ind = find_first_non_zero_index(x);
    if first_non_zero_ind == -1 {
        (0, 0)
    } else {
        let bits = (32 - first_non_zero_ind as usize) * 8
            - x[first_non_zero_ind as usize].leading_zeros() as usize;
        (bits, first_non_zero_ind as usize)
    }
}

fn num_bits_u256_be_and_high_u32(x: [u8; 32]) -> (usize, u32) {
    let (bits, first_non_zero_ind) = num_bits_u256_be_and_first_non_zero_index(x);
    if bits == 0 {
        (0, 0)
    } else {
        let h_u32 = if first_non_zero_ind < 28 {
            u32::from_be_bytes([
                x[first_non_zero_ind],
                x[first_non_zero_ind + 1],
                x[first_non_zero_ind + 2],
                x[first_non_zero_ind + 3],
            ])
        } else if first_non_zero_ind < 29 {
            u32::from_be_bytes([
                0,
                x[first_non_zero_ind],
                x[first_non_zero_ind + 1],
                x[first_non_zero_ind + 2],
            ])
        } else if first_non_zero_ind < 30 {
            u32::from_be_bytes([0, 0, x[first_non_zero_ind], x[first_non_zero_ind + 1]])
        } else {
            u32::from_be_bytes([0, 0, 0, x[first_non_zero_ind]])
        };
        (bits, h_u32)
    }
}
fn reduce_exponent_unsigned(exponent: u32, significand: u32) -> (u32, u32) {
    if exponent == 0 || significand == 0 {
        (0, significand)
    } else if (significand & 0xff0000) != 0 {
        (exponent, significand)
    } else if (significand & 0x00ff00) != 0 {
        (exponent - 1, significand << 8)
    } else if (significand & 0x0000ff) != 0 {
        if exponent >= 2 {
            (exponent - 2, significand >> 16)
        } else {
            (exponent - 1, significand >> 8)
        }
    } else {
        (exponent, significand)
    }
}
fn get_extra_precision_64(
    exponent: u32,
    shifted_significand: u64,
    negative: bool,
) -> BTCDifficulty {
    if shifted_significand == 0 {
        let n_sig = shifted_significand as u32;
        BTCDifficulty::from_parts(0, n_sig, negative).to_lowest_exponent_form()
    } else if exponent == 0 && shifted_significand <= 0xffffffff {
        BTCDifficulty::from_parts(0, 0, negative).to_lowest_exponent_form()
    } else {
        //let shifted_sig_bits  = (64 - shifted_significand.leading_zeros());
        let mut x = shifted_significand;
        let mut new_exponent = exponent;

        if x > 0x7fffffffffffffffu64 {
            x >>= 8;
            new_exponent += 1;
        }

        while x < 0x00ffffffffffffffu64 && new_exponent > 0 {
            x <<= 8;
            new_exponent -= 1;
        }

        if x > 0x7fffffffffffffffu64 {
            x >>= 8;
            new_exponent += 1;
        }

        let mut base_p = (x >> 32) as u64;
        while base_p > 0x7fffff {
            base_p >>= 8;
            new_exponent += 1;
        }
        BTCDifficulty::from_parts(new_exponent, base_p as u32, negative).to_lowest_exponent_form()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BTCDifficulty(u32);
impl BTCDifficulty {
    pub fn get_exponent(&self) -> u32 {
        self.0 >> 24
    }
    pub fn is_negative(&self) -> bool {
        self.0 & DIFFICULTY_NEGATIVE_FLAG != 0
    }
    pub fn get_negative_sign_bit_flag(&self) -> u32 {
        self.0 & DIFFICULTY_NEGATIVE_FLAG
    }
    pub fn get_significand(&self) -> u32 {
        self.0 & 0x007fffff
    }
    pub fn mul_value(&self, value: u32) -> Self {
        if value == 0 {
            return BTCDifficulty(0);
        } else if value == 1 {
            return BTCDifficulty(self.0);
        } else if self.is_zero() {
            return BTCDifficulty(0);
        }

        let significand = self.get_significand();
        let exponent = self.get_exponent();
        let negative = self.is_negative();
        let n_sig = (significand as u64) * (value as u64);
        if n_sig <= 0x007fffff {
            BTCDifficulty::from_parts(exponent, n_sig as u32, negative).to_lowest_exponent_form()
        } else {
            let n_sig = n_sig >> 8;
            let n_exp = exponent + 1;
            BTCDifficulty::from_parts(n_exp, n_sig as u32, negative).to_lowest_exponent_form()
        }
    }
    pub fn div_value(&self, value: u32) -> Self {
        if value == 0 {
            // infinity?
            return BTCDifficulty(0);
        } else if value == 1 {
            return BTCDifficulty(self.0);
        } else if self.is_zero() {
            return BTCDifficulty(0);
        }

        let significand = self.get_significand();
        let exponent = self.get_exponent();
        let negative = self.is_negative();
        let n_sig = ((((significand as u64) << 32) / (value as u64)) >> 32) as u32;
        if n_sig <= 0x007fffff {
            BTCDifficulty::from_parts(exponent, n_sig as u32, negative).to_lowest_exponent_form()
        } else {
            let n_sig = n_sig >> 8;
            let n_exp = exponent + 1;
            BTCDifficulty::from_parts(n_exp, n_sig as u32, negative).to_lowest_exponent_form()
        }
    }
    pub fn new_from_hash(hash: QHash256) -> Self {
        let (n_bits, high_u32) = num_bits_u256_be_and_high_u32(hash);

        let mut n_size = ((n_bits as u32) + 7) / 8;
        let mut n_compact = 0u32;
        if n_size <= 3 {
            n_compact = high_u32 << 8 * (3 - n_size);
        } else {
            n_compact >>= 8;
        }
        // The 0x00800000 bit denotes the sign.
        // Thus, if it is already set, divide the mantissa by 256 and increase the exponent.
        if (n_compact & DIFFICULTY_NEGATIVE_FLAG) != 0 {
            n_compact = n_compact >> 8;
            n_size += 1;
        }
        n_compact |= n_size << 24;
        BTCDifficulty(n_compact)
    }
    pub fn new_from_bits(compact: u32) -> Self {
        Self(compact)
    }
    pub fn get_str(&self) -> String {
        let exponent = self.get_exponent();
        let significand = self.get_significand();
        let negative = self.is_negative();
        format!(
            "exponent: {}, significand: {}, negative: {}",
            exponent, significand, negative
        )
    }
    pub fn is_zero(&self) -> bool {
        self.0 & 0x007fffff == 0
    }
    pub fn from_parts(exponent: u32, significand: u32, negative: bool) -> Self {
        let sign_flag = if negative {
            DIFFICULTY_NEGATIVE_FLAG
        } else {
            0
        };
        Self((exponent << 24) | sign_flag | significand)
    }
    pub fn to_lowest_exponent_form(&self) -> Self {
        let current = self.0;
        if current == 0 || current & 0x007f0000 != 0 || self.get_exponent() == 0 {
            Self(current)
        } else {
            let exponent = self.get_exponent();
            let significand = self.get_significand();
            let sign_bit_flag = self.get_negative_sign_bit_flag();
            let low_16 = significand & 0xffff;

            if significand == 0 {
                Self(sign_bit_flag)
            } else if (low_16 & 0xff00) != 0 {
                if low_16 == (low_16 & 0x7fff) {
                    // it is safe to shift up by 8 bits since it will not disturb the sign bit
                    Self::from_parts(exponent - 1, significand << 8, self.is_negative())
                } else {
                    // we can't shift the significand by 8 bits if it would interfere with the sign bit
                    Self(current)
                }
            } else {
                let low_byte = low_16 & 0xff;
                if low_byte == (low_byte & 0x7f) && exponent >= 2 {
                    // it is safe to shift up by 16 bits since it will not disturb the sign bit
                    Self::from_parts(exponent - 2, significand << 16, self.is_negative())
                } else {
                    Self::from_parts(exponent - 1, significand << 8, self.is_negative())
                }
            }
        }
    }
    pub fn is_greater_than(&self, other: &Self) -> bool {
        if self.is_negative() && !other.is_negative() {
            false
        } else if !self.is_negative() && other.is_negative() {
            true
        } else {
            let (self_exp, self_sig) =
                reduce_exponent_unsigned(self.get_exponent(), self.get_significand());
            let (other_exp, other_sig) =
                reduce_exponent_unsigned(other.get_exponent(), other.get_significand());
            if self_exp > other_exp {
                true
            } else if self_exp < other_exp {
                false
            } else {
                self_sig > other_sig
            }
        }
    }
    pub fn is_equal_to(&self, other: &Self) -> bool {
        self.to_lowest_exponent_form().0 == other.to_lowest_exponent_form().0
    }
    pub fn new_from_bits_0_if_overflow(compact: u32) -> Self {
        let n_size = compact >> 24;
        let n_word = if n_size <= 3 {
            (compact & 0x007fffff) >> (8 * (3 - n_size))
        } else {
            compact & 0x007fffff
        };

        let negative = n_word != 0 && compact & DIFFICULTY_NEGATIVE_FLAG != 0;
        let overflow = n_word != 0
            && ((n_size > 34)
                || (n_word > 0xff && n_size > 33)
                || (n_word > 0xffff && n_size > 32));

        if negative || overflow {
            BTCDifficulty(0)
        } else {
            BTCDifficulty(compact)
        }
    }
    pub fn to_compact_bits(&self) -> u32 {
        self.0
    }
    pub fn is_leq(&self, other: &Self) -> bool {
        !self.is_greater_than(other)
    }
    pub fn is_geq(&self, other: &Self) -> bool {
        self.is_greater_than(other) || self.is_equal_to(other)
    }
    pub fn is_gt(&self, other: &Self) -> bool {
        self.is_greater_than(other)
    }
    pub fn to_adjust_for_next_work(&self, modulated_timespan: i64, retarget_timespan: i64) -> Self {
        let exponent = self.get_exponent();
        let significand = self.get_significand();

        if exponent == 0 {
            let mul_res = (((((significand as u128) * (modulated_timespan as u128)) << 32)
                / (retarget_timespan as u128))
                >> 32) as u32;
            let negative = self.is_negative();
            if mul_res <= 0x007fffff {
                BTCDifficulty::from_parts(exponent, mul_res, negative).to_lowest_exponent_form()
            } else {
                let n_sig = mul_res >> 8;
                let n_exp = exponent + 1;
                BTCDifficulty::from_parts(n_exp, n_sig as u32, negative).to_lowest_exponent_form()
            }
        } else {
            let sig_mul_res = (significand as u64) * (modulated_timespan as u64);
            if sig_mul_res > 0xffff_ffffu64 {
                let mut smr_shifted = sig_mul_res;
                let mut shift_positions = 0;
                while smr_shifted < 0x00ff_ffff_ffff_ffffu64 && shift_positions < 4 {
                    smr_shifted <<= 8;
                    shift_positions += 1;
                }
                let sig_div_1 = smr_shifted / (retarget_timespan as u64);
                get_extra_precision_64(
                    exponent + (4 - shift_positions),
                    sig_div_1,
                    self.is_negative(),
                )
            } else {
                let sig_div_1 = (sig_mul_res << 32) / (retarget_timespan as u64);
                get_extra_precision_64(exponent, sig_div_1, self.is_negative())
            }
        }
    }
    pub fn into_adjust_for_next_work(
        &self,
        modulated_timespan: i64,
        retarget_timespan: i64,
        pow_limit: &Self,
    ) -> u32 {
        let res = self.to_adjust_for_next_work(modulated_timespan, retarget_timespan);
        if res.is_geq(pow_limit) {
            pow_limit.to_compact_bits()
        } else {
            res.to_compact_bits()
        }
    }
}

#[cfg(test)]
mod test {
    use rand::{thread_rng, Rng};

    fn get_extra_precision_128(
        exponent: u32,
        shifted_significand: u128,
        negative: bool,
    ) -> BTCDifficulty {
        if shifted_significand == 0 {
            let n_sig = shifted_significand as u32;
            BTCDifficulty::from_parts(0, n_sig, negative).to_lowest_exponent_form()
        } else if exponent == 0 && shifted_significand <= 0xffffffff {
            BTCDifficulty::from_parts(0, 0, negative).to_lowest_exponent_form()
        } else {
            //let shifted_sig_bits  = (64 - shifted_significand.leading_zeros());
            let mut x = shifted_significand;
            let mut new_exponent = exponent;

            if x > 0x7fffffffffffffffffffffffffffffffu128 {
                x >>= 8;
                new_exponent += 1;
            }

            while x < 0x00ffffffffffffffffffffffffffffffu128 && new_exponent > 0 {
                x <<= 8;
                new_exponent -= 1;
            }

            if x > 0x7fffffffffffffffffffffffffffffffu128 {
                x >>= 8;
                new_exponent += 1;
            }

            let mut base_p = (x >> 64) as u64;
            while base_p > 0x7fffff {
                base_p >>= 8;
                new_exponent += 1;
            }
            BTCDifficulty::from_parts(new_exponent, base_p as u32, negative)
                .to_lowest_exponent_form()
        }
    }

    fn to_adjust_for_next_work_u128(
        x: BTCDifficulty,
        modulated_timespan: i64,
        retarget_timespan: i64,
    ) -> BTCDifficulty {
        let exponent = x.get_exponent();
        let significand = x.get_significand();

        if exponent == 0 {
            let mul_res = (((((significand as u128) * (modulated_timespan as u128)) << 32)
                / (retarget_timespan as u128))
                >> 32) as u32;
            let negative = x.is_negative();
            if mul_res <= 0x007fffff {
                BTCDifficulty::from_parts(exponent, mul_res, negative).to_lowest_exponent_form()
            } else {
                let n_sig = mul_res >> 8;
                let n_exp = exponent + 1;
                BTCDifficulty::from_parts(n_exp, n_sig as u32, negative).to_lowest_exponent_form()
            }
        } else {
            let sig_mul_res = (significand as u64) * (modulated_timespan as u64);
            let sig_div_1 = ((sig_mul_res as u128) << 64) / (retarget_timespan as u128);
            get_extra_precision_128(exponent, sig_div_1, x.is_negative())
        }
    }

    use super::BTCDifficulty;

    fn check_one_128_u64() -> anyhow::Result<()> {
        let s = thread_rng().gen_range(0..=0x7fffffu32);
        let e = thread_rng().gen_range(0..=0x20u32);
        let d1 = BTCDifficulty::from_parts(e, s, false);
        let modulated_timespan = thread_rng().gen_range(0..=0x7fffffffu64);
        let retarget_timespan = thread_rng().gen_range(1..=0xfffffu64);
        let new_v = d1.to_adjust_for_next_work(modulated_timespan as i64, retarget_timespan as i64);
        let old_v =
            to_adjust_for_next_work_u128(d1, modulated_timespan as i64, retarget_timespan as i64);

        assert_eq!(new_v.0, old_v.0);
        Ok(())
    }
    #[test]
    fn test_fuzz_128_64() -> anyhow::Result<()> {
        for i in 0..100 {
            println!("checking {}", i * 1000);
            for _ in 0..1000 {
                check_one_128_u64()?;
            }
        }

        Ok(())
    }
}
