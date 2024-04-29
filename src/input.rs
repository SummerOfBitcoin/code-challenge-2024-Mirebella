use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};

/// Read transaction jsons from mempool folder and build the hashmap
pub(crate) fn read_txs_into_hashmap() -> Result<HashMap<String, String>> {
    let mut result = HashMap::new();

    let path = "mempool";
    for entry in (fs::read_dir(Path::new(path))?).flatten() {
        let path = entry.path();
        if path.is_file() {
            let filename = path
                .file_name()
                .and_then(|f| f.to_str())
                .ok_or_else(|| anyhow!("Filename cannot be empty"))?;
            let temp_path = Path::new(filename).with_extension("");

            let txid = temp_path
                .to_str()
                .ok_or_else(|| anyhow!("Invalid UTF-8 sequence"))?;
            let tx_json =
                fs::read_to_string(&path).map_err(|e| anyhow!("Failed to read file: {e}"))?;
            result.insert(txid.to_string(), tx_json);
        }
    }

    Ok(result)
}
