use anyhow::{anyhow, Result};
use rand::{Rng, thread_rng};
use serde::{Serialize, Deserialize};
use bincode;

#[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum CompressionType {
    None,
    Brotli,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BinaryPacket {
    pub data: Vec<u8>,
    pub compression_type: CompressionType,
}

impl BinaryPacket {
    pub fn new () -> Self {
        BinaryPacket {
            data: vec![],
            compression_type: CompressionType::None,
        }
    }

    pub fn from<T: Sized + Serialize>(data: &T) -> Result<Self> {
        let raw_data = bincode::serialize(data)
            .map_err(|e| anyhow!("{}", e))?;

        Ok(BinaryPacket {
            data: raw_data,
            compression_type: CompressionType::None,
        })
    }
}

pub fn gen_apikey() -> String {
    let mut rng = thread_rng();
    let mut key = "tyb_key_".to_string();
    let nums: Vec<u8> = vec![
        (48..58).collect::<Vec<u8>>(),
        (65..91).collect::<Vec<u8>>(),
        (97..123).collect::<Vec<u8>>(),
    ].concat();
    for _ in 0..64 {
        key.push(nums[rng.gen_range(0..nums.len())] as char);
    }
    key
}

pub mod compression_utils {
    use brotli::{CompressorWriter, Decompressor};
    use std::io::prelude::*;
    use anyhow::{anyhow, Result};

    use super::{BinaryPacket, CompressionType};

    pub fn decompress(packet: &mut BinaryPacket) -> Result<()> {
        match packet.compression_type {
            CompressionType::Brotli => decompress_brotli(packet),
            _ => Ok(()),
        }
    }

    pub fn compress_brotli (packet: &mut BinaryPacket) -> Result<()> {
        if packet.compression_type != CompressionType::None {
            return Err(anyhow!("Data is already compressed."));
        }

        let mut encoder = CompressorWriter::new(Vec::new(), 4096, 11, 22);
        encoder.write_all(&packet.data)?;
        let compressed_data = encoder.into_inner();
        packet.data = compressed_data;
        packet.data.shrink_to_fit();
        packet.compression_type = CompressionType::Brotli;
        Ok(())
    }

    pub fn decompress_brotli(packet: &mut BinaryPacket) -> Result<()>{
        if packet.compression_type != CompressionType::Brotli {
            return Err(anyhow!("Data is not brotli compressed."));
        }

        let mut decompressed_data = Vec::new();
        let mut decompressor = Decompressor::new(packet.data.as_slice(), 4096_000);
        decompressor.read_to_end(&mut decompressed_data)?;

        packet.data = decompressed_data;
        packet.compression_type = CompressionType::None;

        Ok(())
    }
}
