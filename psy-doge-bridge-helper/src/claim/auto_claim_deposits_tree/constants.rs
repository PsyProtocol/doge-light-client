use doge_light_client::common_types::QHash256;

use crate::utils::sha256_zero_hashes::SHA256_ZERO_HASHES;

pub const AUTO_CLAIM_DEPOSITS_TREE_HEIGHT: usize = 32;
pub const AUTO_CLAIM_DEPOSITS_TREE_EMPTY_ROOT: QHash256 = SHA256_ZERO_HASHES[AUTO_CLAIM_DEPOSITS_TREE_HEIGHT];
