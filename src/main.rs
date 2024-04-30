mod block;
mod input;
mod mine;
mod output;
mod validation;

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::block::{create_block, create_coinbase_transaction};
use crate::mine::mine;
use crate::output::write_block_to_file;
use crate::validation::Transaction;

fn main() -> Result<()> {
    // input
    let txs = input::read_txs_into_hashmap()?;
    println!("All tx count: {:?}", txs.len());

    // validation
    let validated_txs_hashmap = validation::validate_all_transactions(txs);
    let mut validated_txs: Vec<Transaction> = validated_txs_hashmap.into_values().collect();
    println!("Validated tx count: {:?}", validated_txs.len());

    // block
    let previous_block_hash =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let bits_u256 = primitive_types::U256::from(
        "0000ffff00000000000000000000000000000000000000000000000000000000",
    );

    // Add to validated tx the coinbase transaction
    let block_reward = 50;
    let miner_address = "my_miner_address"; // TODO
    let coinbase_tx = create_coinbase_transaction(block_reward, miner_address);
    validated_txs.insert(0, coinbase_tx);

    let block = create_block(validated_txs, previous_block_hash, time, bits_u256);
    println!("Block header (before mining): {:?}", block.header);
    println!("Block tx count: {:?}", block.transactions.len());

    // mine
    let mined_block = mine(block, bits_u256);
    println!("Block was successfully mined!");
    println!("Block header (after mining): {:?}", &mined_block.header);

    // write to file
    write_block_to_file(&mined_block.header, &mined_block.transactions, "output.txt")?;

    Ok(())
}
