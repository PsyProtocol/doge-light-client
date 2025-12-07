use doge_light_client::hash::sha256_impl::hash_impl_sha256_two_to_one_bytes;

fn generate_n_zero_hashes(n: usize) -> Vec<[u8; 32]> {
    let mut hashes = Vec::with_capacity(n);
    let mut current_hash = [0u8; 32];
    hashes.push(current_hash);
    for _ in 1..n {
        current_hash = hash_impl_sha256_two_to_one_bytes(&current_hash, &current_hash);
        hashes.push(current_hash);
    }
    hashes
}
fn format_zero_hashes_as_rust_array(hashes: &[[u8; 32]]) -> String {
    let mut result = String::from("const ZERO_HASHES: [[u8; 32]; ");
    result.push_str(&hashes.len().to_string());
    result.push_str("] = [\n");
    for hash in hashes {
        result.push_str("    [");
        for (i, byte) in hash.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!("0x{:02x}", byte));
        }
        result.push_str("],\n");
    }
    result.push_str("];\n");
    result
}
fn main() {
    let zero_hashes = generate_n_zero_hashes(64);
    let formatted = format_zero_hashes_as_rust_array(&zero_hashes);
    println!("{}", formatted);
}