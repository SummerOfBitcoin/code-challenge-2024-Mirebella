use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::mine;
use crate::validation::{Input, Output, PrevOut, Transaction};

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: Header,
    pub transactions: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    version: u32,
    previous_block_hash: String,
    merkle_root: String,
    time: u32,
    pub(crate) bits: u32,
    pub nonce: u32,
}
impl Header {
    pub(crate) fn to_hex(&self) -> Result<String> {
        let mut header_bytes = Vec::with_capacity(80);

        header_bytes.write_u32::<LittleEndian>(self.version)?;
        header_bytes.extend_from_slice(&hex::decode(&self.previous_block_hash)?);
        header_bytes.extend_from_slice(&hex::decode(&self.merkle_root)?);
        header_bytes.write_u32::<LittleEndian>(self.time)?;
        header_bytes.write_u32::<LittleEndian>(self.bits)?;
        header_bytes.write_u32::<LittleEndian>(self.nonce)?;

        Ok(hex::encode(header_bytes))
    }
}

pub fn create_block(
    transactions: Vec<Transaction>,
    previous_block_hash: String,
    time: u32,
    bits_decompressed: primitive_types::U256,
) -> Block {
    let merkle_root = calculate_merkle_root(&transactions);
    let bits_compressed = mine::compress_target(bits_decompressed);
    let header = create_header(previous_block_hash, merkle_root, time, bits_compressed);
    Block {
        header,
        transactions,
    }
}

fn create_header(previous_block_hash: String, merkle_root: String, time: u32, bits: u32) -> Header {
    Header {
        version: 4,
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
        .map(|tx| hex::encode(sha256(serde_json::to_string(tx).unwrap().as_bytes())))
        .collect::<Vec<_>>();
    hex::encode(sha256(hashes.join("").as_bytes()))
}

pub(crate) fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

pub(crate) fn double_sha256(data: &[u8]) -> Vec<u8> {
    let first = sha256(data);
    sha256(&first)
}

pub(crate) fn create_coinbase_transaction(reward: u64, miner_address: &str) -> Transaction {
    Transaction {
        version: 1,
        locktime: 0,
        vin: vec![Input {
            txid: String::new(), // No input transaction (new coins)
            vout: 0,
            prevout: PrevOut {
                scriptpubkey: String::new(),
                scriptpubkey_asm: String::new(),
                scriptpubkey_type: String::new(),
                scriptpubkey_address: String::new(),
                value: 0,
            },
            scriptsig: String::new(), // No signature script for coinbase
            scriptsig_asm: String::new(),
            witness: vec![],
            is_coinbase: true,
            sequence: 0xFFFFFFFF,
        }],
        vout: vec![Output {
            scriptpubkey: format!(
                "OP_DUP OP_HASH160 {} OP_EQUALVERIFY OP_CHECKSIG",
                miner_address
            ),
            scriptpubkey_asm: String::new(),
            scriptpubkey_type: "pubkeyhash".to_string(),
            scriptpubkey_address: miner_address.to_string(),
            value: reward, // Block reward
        }],
    }
}
