mod input;

use anyhow::Result;

fn main() -> Result<()> {
    let txs = input::read_txs_into_hashmap()?;
    println!("Transaction count: {:?}", txs.len());

    Ok(())
}
