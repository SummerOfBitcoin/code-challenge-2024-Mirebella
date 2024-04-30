use crate::block::double_sha256;
use crate::block::{Block, Header};

use anyhow::Result;

pub fn mine(mut block: Block, target_difficulty_u256: primitive_types::U256) -> Result<Block> {
    loop {
        let hash = calculate_block_hash(&block.header)?;
        let hash_u256 = primitive_types::U256::from_big_endian(&hash);

        if hash_u256 < target_difficulty_u256 {
            let mut target_difficulty_bytes = [0; 32];
            target_difficulty_u256.to_big_endian(&mut target_difficulty_bytes);

            println!("hash:   {}", hex::encode(hash));
            println!("target: {}", hex::encode(target_difficulty_bytes));

            return Ok(block);
        }
        block.header.nonce += 1;
    }
}

pub(crate) fn compress_target(target: primitive_types::U256) -> u32 {
    let mut size = (target.bits() + 7) / 8; // Calculate size in bytes
    let mut compact = if size <= 3 {
        // If the target is small enough to fit in 3 bytes
        target.low_u32() << (8 * (3 - size))
    } else {
        // Shift the target right to fit it into 3 bytes
        let shift_bits = 8 * (size - 3);
        (target >> shift_bits).low_u32()
    };

    // If the compact form is > 0x007fffff, increment size and shift right by another 8 bits
    if compact & 0x00800000 != 0 {
        compact >>= 8;
        size += 1;
    }

    // The compact format: the first byte is the size, next three are the coefficient
    ((size as u32) << 24) | (compact & 0x00ffffff)
}

fn calculate_block_hash(header: &Header) -> Result<Vec<u8>> {
    let header_bytes = hex::decode(header.to_hex()?)?;
    let mut double_hash = double_sha256(&header_bytes);
    double_hash.reverse();
    Ok(double_hash)
}
