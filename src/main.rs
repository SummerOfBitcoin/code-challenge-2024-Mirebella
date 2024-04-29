mod block;
mod input;
mod mine;
mod validation;
mod write_to_file;

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::block::{create_block, create_coinbase_transaction};
use crate::mine::mine;

fn main() -> Result<()> {
    // input
    let txs = input::read_txs_into_hashmap()?;
    println!("All tx count: {:?}", txs.len());

    // validation
    let mut validated_txs = validation::validate_all_transactions(txs);
    println!("Validated tx count: {:?}", validated_txs.len());

    // block
    let previous_block_hash =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let bits_u256 = primitive_types::U256::from(
        "0000ffff00000000000000000000000000000000000000000000000000000000",
    );

    // Add to validated tx the coinbase transaction
    let block_reward = 50;
    let miner_address = "my_miner_address";
    validated_txs.push(create_coinbase_transaction(block_reward, miner_address));

    let block = create_block(validated_txs, previous_block_hash, time, bits_u256);
    println!("Block header (before mining): {:?}", block.header);
    println!("Block tx count: {:?}", block.transactions.len());

    // mine
    let header = mine(block, bits_u256);
    println!("Block was successfully mined!");
    println!("Block header (after mining): {:?}", header);

    Ok(())
}
