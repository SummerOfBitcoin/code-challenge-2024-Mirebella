use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::validation::convert_json_to_tx;
use anyhow::{anyhow, Result};

/// Read transaction jsons from mempool folder and build the hashmap
pub(crate) fn read_txs_into_hashmap() -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();

    let path = "mempool";
    for entry in (fs::read_dir(Path::new(path))?).flatten() {
        let path = entry.path();
        if path.is_file() {
            let tx_json =
                fs::read_to_string(&path).map_err(|e| anyhow!("Failed to read file: {e}"))?;

            if let Ok(tx) = convert_json_to_tx(&tx_json) {
                let txid = tx.id()?;
                result.insert(txid, tx_json);
            }
        }
    }

    Ok(result)
}
