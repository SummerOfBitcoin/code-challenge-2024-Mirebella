use hex::encode;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use crate::validation::Transaction;

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    version: i32,
    previous_block_hash: String,
    merkle_root: String,
    time: u64,
    bits: u32,
    pub nonce: u64,
}

pub fn create_block(transactions: Vec<Transaction>, previous_block_hash: String, time: u64, bits: u32) -> Block {
    let merkle_root = calculate_merkle_root(&transactions);
    let header = create_header(previous_block_hash, merkle_root, time, bits);
    Block {
        header,
        transactions,
    }
}

fn create_header(previous_block_hash: String, merkle_root: String, time: u64, bits: u32) -> Header {
    Header {
        version: 1,
        previous_block_hash,
        merkle_root,
        time,
        bits,
        nonce: 0,
    }
}

fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    let hashes = transactions
        .iter()
        .map(|tx| encode(sha256(serde_json::to_string(tx).unwrap().as_bytes())))
        .collect::<Vec<_>>();
    encode(sha256(hashes.join("").as_bytes()))
}

fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}