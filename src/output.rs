use crate::block::Header;
use crate::validation::Transaction;

use std::fs::File;
use std::io::{BufWriter, Write};

use anyhow::Result;

pub(crate) fn write_block_to_file(
    header: &Header,
    transactions: &[Transaction],
    path: &str,
) -> Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{}", header.to_hex()?)?;

    if let Some(coinbase) = transactions.first() {
        writeln!(writer, "{}", serde_json::to_string(coinbase)?)?;
        for transaction in transactions {
            writeln!(writer, "{}", transaction.id()?)?; // TODO id() correct?
        }
    }

    Ok(())
}
