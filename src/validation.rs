use std::collections::HashMap;

use serde::{Deserialize, Serialize};

const TOTAL_MONEY_CAP: u64 = 21_000_000 * 100_000_000;
const MAX_BLOCK_SIZE: usize = 1_000_000;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Transaction {
    pub(crate) version: i32,
    pub(crate) locktime: u32,
    pub(crate) vin: Vec<Input>,
    pub(crate) vout: Vec<Output>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Input {
    pub(crate) txid: String,
    pub(crate) vout: u32,
    pub(crate) prevout: PrevOut,
    pub(crate) scriptsig: String,
    pub(crate) scriptsig_asm: String,
    pub(crate) witness: Vec<String>,
    pub(crate) is_coinbase: bool,
    pub(crate) sequence: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PrevOut {
    pub(crate) scriptpubkey: String,
    pub(crate) scriptpubkey_asm: String,
    pub(crate) scriptpubkey_type: String,
    pub(crate) scriptpubkey_address: String,
    pub(crate) value: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Output {
    pub(crate) scriptpubkey: String,
    pub(crate) scriptpubkey_asm: String,
    pub(crate) scriptpubkey_type: String,
    pub(crate) scriptpubkey_address: String,
    pub(crate) value: u64,
}

pub(crate) fn validate_all_transactions(txs: HashMap<String, String>) -> Vec<Transaction> {
    let mut valid_txs = Vec::new();
    let outputs_hashmap = create_output_hashmap(&txs);
    for (txid, tx_json) in &txs {
        if let Some(tx) = is_transaction_valid(txid, &tx_json, &outputs_hashmap) {
            valid_txs.push(tx)
        }
    }

    valid_txs
}

fn is_transaction_valid(
    tx_id: &str,
    tx_json: &&String,
    output_hashmap: &HashMap<String, String>,
) -> Option<Transaction> {
    // Check syntactic correctness
    let maybe_tx = is_valid_syntax(tx_json);
    maybe_tx.as_ref()?;
    let tx = maybe_tx.unwrap();

    // Make sure neither in or out lists are empty
    if !is_valid_in_and_out_txs_lists_are_not_empty(&tx) {
        return None;
    }

    // Size in bytes <= MAX_BLOCK_SIZE
    if !is_valid_max_block_size_correct(tx_json) {
        return None;
    }

    // Each output value, as well as the total, must be in legal money range
    if !is_valid_check_output_and_total_money_range(&tx) {
        return None;
    }

    // Make sure none of the inputs have hash=0, n=-1 (coinbase transactions)
    if !is_valid_check_hash_and_coinbase(&tx) {
        return None;
    }

    // Check that nLockTime <= INT_MAX[1], size in bytes >= 100[2], and sig opcount <= 2[3]
    if !is_valid_check_n_lock_time_size_sign_opcount(tx_json, &tx) {
        return None;
    }

    // Reject "nonstandard" transactions: scriptSig doing anything other than pushing numbers on the stack, or scriptPubkey not matching the two usual forms
    if !is_valid_reject_nonstandard_txs(&tx) {
        return None;
    }

    // Reject if transaction fee (defined as sum of input values minus sum of output values) would be too low to get into an empty block
    if !is_valid_check_tx_fee(&tx) {
        return None;
    }

    // For each input, if the referenced output exists in any other tx in the pool, reject this transaction.
    if !is_valid_check_if_output_exists_in_other_tx(tx_id, &tx, output_hashmap) {
        return None;
    }

    // Reject if the sum of input values < sum of output values
    if !is_valid_sum_of_inputs_bigger_than_outputs(&tx) {
        return None;
    }

    Some(tx)
}

fn is_valid_sum_of_inputs_bigger_than_outputs(tx: &Transaction) -> bool {
    let total_input_value: u64 = tx.vin.iter().map(|input| input.prevout.value).sum();
    let total_output_value: u64 = tx.vout.iter().map(|output| output.value).sum();

    total_input_value >= total_output_value
}

fn is_valid_check_if_output_exists_in_other_tx(
    current_tx_id: &str,
    current_tx: &Transaction,
    output_references: &HashMap<String, String>,
) -> bool {
    for vin in &current_tx.vin {
        let key = format!("{}:{}", vin.txid, vin.vout);
        match output_references.get(&key) {
            None => {}
            Some(matching_txid) => {
                if matching_txid != current_tx_id {
                    return false;
                }
            }
        }
    }

    true
}

fn create_output_hashmap(txs: &HashMap<String, String>) -> HashMap<String, String> {
    let mut output_hashmap = HashMap::new();
    for (txid, tx_json) in txs {
        if let Ok(tx) = serde_json::from_str::<Transaction>(tx_json) {
            for input in &tx.vin {
                let key = format!("{}:{}", input.txid, input.vout);
                output_hashmap.insert(key, txid.clone());
            }
        }
    }
    output_hashmap
}

fn is_valid_check_tx_fee(tx: &Transaction) -> bool {
    // Calculate the total input value
    let total_input_value: u64 = tx.vin.iter().map(|input| input.prevout.value).sum();

    // Calculate the total output value
    let total_output_value: u64 = tx.vout.iter().map(|output| output.value).sum();

    let tx_fee = total_input_value.saturating_sub(total_output_value);

    // Assume min fee is 1 sat
    tx_fee >= 1
}

fn is_valid_reject_nonstandard_txs(tx: &Transaction) -> bool {
    // Check each input's scriptSig
    for input in &tx.vin {
        let scriptsig_asm = &input.scriptsig_asm;
        // Check if scriptSig does more than pushing numbers (very basic check)
        if !scriptsig_asm.is_empty()
            && !scriptsig_asm.starts_with("OP_0")
            && !scriptsig_asm.contains("OP_PUSHBYTES")
        {
            return false;
        }
    }

    // Check each output's scriptPubKey
    for output in &tx.vout {
        let asm = &output.scriptpubkey_asm;

        // Check for standard P2PKH and P2SH formats
        let matches_p2pkh_format =
            asm.starts_with("OP_DUP OP_HASH160") && asm.ends_with("OP_EQUALVERIFY OP_CHECKSIG");
        let matches_p2sh_format = asm.starts_with("OP_HASH160") && asm.ends_with("OP_EQUAL");

        if !(matches_p2pkh_format || matches_p2sh_format) {
            return false;
        }
    }

    true
}

fn is_valid_check_n_lock_time_size_sign_opcount(tx_json: &&String, tx: &Transaction) -> bool {
    // Check nLockTime <= INT_MAX
    let locktime = tx.locktime as u64;
    if locktime > i32::MAX as u64 {
        return false;
    }

    // Check size in bytes >= 100
    if tx_json.as_bytes().len() < 100 {
        return false;
    }

    // Count signature operations
    for input in &tx.vin {
        let count_op_checksig = input.scriptsig_asm.matches("OP_CHECKSIG").count();
        let count_op_checksigverify = input.scriptsig_asm.matches("OP_CHECKSIGVERIFY").count();

        if count_op_checksig + count_op_checksigverify > 2 {
            return false;
        }
    }

    true
}

fn is_valid_check_output_and_total_money_range(tx: &Transaction) -> bool {
    let mut total_input_value = 0u64;
    let mut total_output_value = 0u64;

    // Sum all output values and check individual outputs
    for output in &tx.vout {
        total_output_value += output.value;
        if total_output_value >= TOTAL_MONEY_CAP {
            println!("Output value exceeds the total money cap.");
            return false;
        }
    }

    // Sum all input values and check individual inputs
    for input in &tx.vin {
        total_input_value += input.prevout.value;
        if total_input_value >= TOTAL_MONEY_CAP {
            println!("Input value exceeds the total money cap.");
            return false;
        }
    }

    true
}

fn is_valid_check_hash_and_coinbase(tx: &Transaction) -> bool {
    for input in &tx.vin {
        if input.is_coinbase {
            return false;
        }
    }

    true
}

fn is_valid_max_block_size_correct(tx_json: &str) -> bool {
    tx_json.as_bytes().len() <= MAX_BLOCK_SIZE
}

fn is_valid_in_and_out_txs_lists_are_not_empty(tx: &Transaction) -> bool {
    !tx.vin.is_empty() && !tx.vout.is_empty()
}

fn is_valid_syntax(tx_json: &str) -> Option<Transaction> {
    serde_json::from_str::<Transaction>(tx_json).ok()
}
