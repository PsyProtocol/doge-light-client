use std::str::FromStr;

use doge_light_client::doge::{address::BTCAddress160, transaction::BTCTransaction};
use qed_doge_bridge_helper::tx_template::is_bridge_desposit_output_v1_for_user;

fn check_deposit_tx(
    tx: &[u8],
    solana_public_key: &[u8],
    bridge_public_key_hash: &[u8],
    output_index: usize,
) -> bool {
    match BTCTransaction::get_output_skip_decode(tx, 0, output_index) {
        Ok((version, locktime, output)) => {
            if version != 1 && version != 2 {
                return false;
            } else if locktime != 0 {
                return false;
            }

            is_bridge_desposit_output_v1_for_user(
                &output,
                &solana_public_key,
                &bridge_public_key_hash,
            )
        }
        Err(_) => false,
    }
}

fn main() {
    let tx = hex_literal::hex!(
        "02000000025136955474FD35B4F19064276E90E6AD7AD6732F6BF99F1E3130B9545F01CB37000000006A47304402206DD8D414BBCEB14146F58D9559159DB9557E350D2E3DB9CA06318B0AD8B10C4E02203D9BFFA49904EF779FAAE8A4DE4FC10ED6E0B7F5D5E1567999F08EB49A032FC60121037175782B4E0DFEF8BDB35F29A9E1CDBFF913B8300D7F33B6E041C862C015EB35FFFFFFFF0D29906C5646473F3CC48E8B9892FE47AE691F0B60B800DFA8B095C45590DC51000000006A4730440220140B3EEC07DC4A04D05609EDB845EFEDA748CCB1CA17FFFAC3A4B06A7DD378800220663380DD7FEC3E897B0F6ABC56BBBBACEADAA31229719C65584C8216A6DF6D4E0121037175782B4E0DFEF8BDB35F29A9E1CDBFF913B8300D7F33B6E041C862C015EB35FFFFFFFF024EC0400BF35A000017A914B1B4C196B398C9ACB414DB8E7383930C7639D6A787EF48BAD08C0E00001976A9145782169A69A599E092C2DAB929056773ABB50C9088AC00000000"
    );

    let solana_public_key =
        hex_literal::hex!("e83c24b97aeadd8de838b7c040347ac9e821a103c38b2999a7989f7a6181e0d8");

    let tt = BTCTransaction::from_bytes(&tx).unwrap();
    let ttl = tt.to_bytes().len();

    println!("ttl: {}", ttl);


    let bridge_public_key_hash = BTCAddress160::from_str("nidKRv4eeRaLzngA34r8epXFNnJS54GJ1R")
        .unwrap()
        .address;

    let res = check_deposit_tx(&tx, &solana_public_key, &bridge_public_key_hash, 0);

    println!("res: {}", res);

    //is_bridge_desposit_output_v1_for_user(output, &solana_public_key, &bridge_public_key_hash)
}
