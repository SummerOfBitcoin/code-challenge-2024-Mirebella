mod input;
mod validation;

use anyhow::Result;

fn main() -> Result<()> {
    // input
    let txs = input::read_txs_into_hashmap()?;
    println!("Transaction count: {:?}", txs.len());

    // validation
    let validated_txs = validation::validate_all_transactions(txs)?;
    println!("Transaction count: {:?}", validated_txs.len());

    Ok(())
}
