use doge_light_client::{common_types::QHash256, hash::sha256_impl::hash_impl_sha256_bytes};

pub fn hash_two_to_one_buffer(buf: &mut [u8; 64], left: &QHash256, right: &QHash256) -> QHash256 {
    buf[0..32].copy_from_slice(left);
    buf[32..64].copy_from_slice(right);
    hash_impl_sha256_bytes(buf)
}
pub fn hash_three_to_one_buffer(
    buf: &mut [u8; 64],
    a: &QHash256,
    b: &QHash256,
    c: &QHash256,
) -> QHash256 {
    let bc = hash_two_to_one_buffer(buf, b, c);
    hash_two_to_one_buffer(buf, a, &bc)
}

pub fn linear_hash_stack_2d_push_single(
    previous_groups_linear_hash: &QHash256,
    tip_group_hash: &QHash256,
    item: &QHash256,
) -> QHash256 {
    let mut buf = [0u8; 64];
    hash_three_to_one_buffer(&mut buf, previous_groups_linear_hash, tip_group_hash, item)
}
pub fn linear_hash_stack_2d_push_single_verify(
    current_2d_list_hash: &QHash256,
    previous_groups_linear_hash: &QHash256,
    tip_group_hash: &QHash256,
    item: &QHash256,
) -> QHash256 {
    let mut buf = [0u8; 64];
    assert_eq!(
        current_2d_list_hash,
        &hash_two_to_one_buffer(&mut buf, previous_groups_linear_hash, tip_group_hash,)
    );

    hash_three_to_one_buffer(&mut buf, previous_groups_linear_hash, tip_group_hash, item)
}
pub fn linear_hash_stack_2d_pop_single_verify(
    current_2d_list_hash: &QHash256,
    previous_groups_linear_hash: &QHash256,
    tip_group_hash_without_item: &QHash256,
    item: &QHash256,
) -> QHash256 {
    let mut buf = [0u8; 64];

    assert_eq!(
        current_2d_list_hash,
        &hash_three_to_one_buffer(
            &mut buf,
            previous_groups_linear_hash,
            tip_group_hash_without_item,
            item
        )
    );
    hash_two_to_one_buffer(
        &mut buf,
        previous_groups_linear_hash,
        tip_group_hash_without_item,
    )
}
pub fn linear_hash_list(
    list: &[QHash256],
) -> QHash256 {
    let mut current_hash = QHash256::default();
    let mut buf = [0u8; 64];
    for group_hash in list.iter() {
        current_hash = hash_two_to_one_buffer(&mut buf, group_hash, &current_hash);
    }
    current_hash
}
pub fn linear_hash_2d_list(
    list_of_groups: &[Vec<QHash256>],
) -> QHash256 {
    let group_hashes: Vec<QHash256> = list_of_groups
        .iter()
        .map(|group| linear_hash_list(group))
        .collect();
    linear_hash_list(&group_hashes)
}