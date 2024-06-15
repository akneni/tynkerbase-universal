#![allow(unused)]

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use ring::rand::{SecureRandom, SystemRandom};
use anyhow::{anyhow, Result};
use rsa::pkcs1::EncodeRsaPublicKey;
use rsa::{RsaPrivateKey, RsaPublicKey};
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
    pub is_encrypted: bool,
    pub compression_type: CompressionType,
    aad: Vec<u8>,
    nonce: Option<Vec<u8>>,
}

impl BinaryPacket {
    pub fn from<T: Sized + Serialize>(data: &T) -> Result<Self> {
        let aad = b"Insert easter egg here.".to_vec();
        let raw_data = bincode::serialize(data)
            .map_err(|e| anyhow!("{}", e))?;

        Ok(BinaryPacket {
            data: raw_data,
            is_encrypted: false,
            compression_type: CompressionType::None,
            aad: aad,
            nonce: None,
        })
    }

    pub fn mem_size(&self) -> usize {
        let mut size = self.data.len() + self.aad.len() + 1;
        if let Some(ref n) = self.nonce {
            size += n.len();
        }
        size
    }
}

pub struct AesKeys {
    key: [u8; 32]
}

impl AesKeys {
    pub fn new() -> Self {
        let rng = SystemRandom::new();
        let mut key = [0_u8; 32];
        rng.fill(&mut key);
        
        AesKeys{
            key: key
        }
    }

    pub fn from (key: [u8; 32]) -> Self {
        AesKeys{
            key: key
        }
    }

    pub fn encrypt(&self, msg: &mut BinaryPacket) -> Result<()> {
        let rng = SystemRandom::new();

        let mut in_out = if !msg.is_encrypted {
            msg.data.clone()
        }
        else {
            return Err(anyhow!("message is already encrypted"));
        };

        // Prepare the key
        let unbound_key = match UnboundKey::new(&AES_256_GCM, &self.key) {
            Ok(k) => k,
            Err(e) => return Err(anyhow!("Failed to create new key: {}", e)),
        };
        let less_safe_key = LessSafeKey::new(unbound_key);
    
        // Generate a unique nonce for this operation
        let mut nonce_bytes = vec![0u8; 12]; // 96 bits for AES-GCM nonce
        rng.fill(&mut nonce_bytes).unwrap();
        msg.nonce = Some(nonce_bytes.clone());
    
        let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes).unwrap();
    
        // The buffer where the in-place operation will happen
        let tag = less_safe_key.seal_in_place_separate_tag(nonce, Aad::from(msg.aad.clone()), &mut in_out).unwrap();
    
        // Append the tag to the ciphertext
        in_out.extend_from_slice(tag.as_ref());
    

        msg.data = in_out;
        msg.is_encrypted = true;
        Ok(())
    }

    pub fn decrypt(&self, msg: &mut BinaryPacket) -> Result<()> {
        // Prepare the key
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key).unwrap();
        let less_safe_key = LessSafeKey::new(unbound_key);

        let ciphertext = if msg.is_encrypted {
            msg.data.clone()
        }
        else {
            return Err(anyhow!("no ciphertext in message"));
        };
        let nonce = if let Some(nonce) = msg.nonce.clone() {
            msg.nonce = None;
            nonce
        }
        else {
            return Err(anyhow!("no nonce in message"));
        };
        let aad = msg.aad.clone();

    
        // Convert the nonce back to a `Nonce` type
        let nonce = Nonce::try_assume_unique_for_key(nonce.as_slice()).unwrap();
    
        // Separate the actual ciphertext from the tag appended at the end
        // The tag is the last 16 bytes of the input provided for AES-256 GCM
        let tag_len = AES_256_GCM.tag_len();
        let (ciphertext, tag) = ciphertext.split_at(ciphertext.len() - tag_len);
    
        let mut in_out = ciphertext.to_vec();
        // Append the tag to be able to verify it during decryption
        in_out.extend_from_slice(tag);
    
        // Decrypt in place, which also verifies the tag
        let plaintext = less_safe_key.open_in_place(nonce, Aad::from(aad), &mut in_out).unwrap();
        msg.data = plaintext.to_vec();
        msg.is_encrypted = false;
        Ok(())
    }
}


#[derive(Debug)]
pub struct RsaKeys {
    pub priv_key: RsaPrivateKey,
    pub pub_key: RsaPublicKey,
}

impl RsaKeys {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits)
            .expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);
        RsaKeys {
            priv_key: priv_key,
            pub_key: pub_key
        }
    }

    pub fn get_pub_key_binary(&self) -> Vec<u8> {
        let pub_key_pem = self.pub_key.to_pkcs1_der()
            .expect("unable to extract public key binary");
        pub_key_pem.as_bytes().to_vec()
    }
}

pub mod rsa_utils {
    use rand::thread_rng;
    use rsa::{RsaPrivateKey, RsaPublicKey};
    use anyhow::{anyhow, Result};

    use super::BinaryPacket;

    pub const ENCRYPT_CHUNK_SIZE: usize = 245;
    pub const DECRYPT_CHUNK_SIZE: usize = 256;

    pub fn encrypt(packet: &mut BinaryPacket, key: &RsaPublicKey) -> Result<()> {
        if packet.is_encrypted {
            return Err(anyhow!("packet is already encrypted."));
        }
        let mut rng = thread_rng();

        let mut enc_data: Vec<u8> = vec![];

        for d in packet.data.chunks(ENCRYPT_CHUNK_SIZE) {
            let mut enc_buff = key.encrypt(&mut rng, rsa::Pkcs1v15Encrypt, d)
                .map_err(|e| anyhow!("{}", e))?;
            enc_data.append(&mut enc_buff);
        }

        packet.data = enc_data;
        packet.is_encrypted = true;

        Ok(())
    }

    pub fn decrypt(packet: &mut BinaryPacket, key: &RsaPrivateKey) -> Result<()> {
        if !packet.is_encrypted {
            return Err(anyhow!("packet is not encrypted."));
        }

        let mut plaintext: Vec<u8> = vec![];

        for d in packet.data.chunks(DECRYPT_CHUNK_SIZE) {
            let mut dec_buffer = key.decrypt(rsa::Pkcs1v15Encrypt, d)
                .map_err(|e| anyhow!("Error decrypting chunk: {}", e))?;
            plaintext.append(&mut dec_buffer);
        }

        packet.data = plaintext;
        packet.is_encrypted = false;

        Ok(())
    }

}


pub mod compression_utils {
    use brotli::{CompressorWriter, Decompressor};
    use std::io::prelude::*;
    use anyhow::{anyhow, Result};

    use super::{BinaryPacket, CompressionType};

    pub fn compress_brotli (packet: &mut BinaryPacket) -> Result<()> {
        if packet.is_encrypted {
            return Err(anyhow!("Error, compressing encrypted data is useless."));
        }
        else if packet.compression_type != CompressionType::None {
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
        if packet.is_encrypted {
            return Err(anyhow!("Error, decrypt packet before decompressing"));
        }
        else if packet.compression_type != CompressionType::Brotli {
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
