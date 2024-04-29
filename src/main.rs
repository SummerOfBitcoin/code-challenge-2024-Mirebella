mod block;
mod input;
mod validation;

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::block::create_block;

fn main() -> Result<()> {
    // input
    let txs = input::read_txs_into_hashmap()?;
    println!("All tx count: {:?}", txs.len());

    // validation
    let validated_txs = validation::validate_all_transactions(txs);
    println!("Validated tx count: {:?}", validated_txs.len());

    // block
    let previous_block_hash =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let bits: u32 = 0x0000ffff; // target difficulty

    let block = create_block(validated_txs, previous_block_hash, time, bits);
    println!("Block header: {:?}", block.header);
    println!("Block tx count: {:?}", block.transactions.len());

    Ok(())
}
