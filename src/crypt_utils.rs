use anyhow::{anyhow, Result};
use rand::{Rng, thread_rng};
use serde::{Serialize, Deserialize};
use bincode;
use sha2::{Digest, Sha512};
use hex;
use rpassword::read_password;
use std::io::{self, Write};


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

pub fn gen_apikey(pass_sha384: &str, salt: &str) -> String {
    let mut combined_key = bincode::serialize(pass_sha384).unwrap();
    let mut salt = bincode::serialize(salt).unwrap();
    combined_key.append(&mut salt);

    let mut hasher = Sha512::new();
    hasher.update(combined_key);
    let key = hasher.finalize();

    let key = hex::encode(key);
    let key = format!("tyb_key_{}", &key);
    key
}

pub fn gen_salt() -> String {
    let mut rng = thread_rng();
    let mut key = "tyb_salt_".to_string();
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

pub fn prompt(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

pub fn prompt_secret(message: &str) -> String {
    print!("{}", message);
    io::stdout().flush().unwrap();
    let password = read_password().expect("Failed to read secret line");
    password.trim().to_string()
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


pub mod hash_utils {
    use sha2::{Digest, Sha256, Sha384, Sha512};
    use hex;

    pub fn sha256(data: impl AsRef<[u8]>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn sha384(data: impl AsRef<[u8]>) -> String {
        let mut hasher = Sha384::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn sha512(data: impl AsRef<[u8]>) -> String {
        let mut hasher = Sha512::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

pub mod aes_utils {
    use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
    use ring::rand::{SecureRandom, SystemRandom};
    use serde::{Serialize, Deserialize};
    use anyhow::{anyhow, Result};

    use super::hash_utils;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AesMsg {
        pub data: Vec<u8>,
        pub aad: Vec<u8>,
        pub nonce: Option<Vec<u8>>,
        pub is_encrypted: bool,
    }
    
    impl AesMsg {
        const AAD: &'static str = "1523 Elizabeth Ave #130, Charlotte, NC 28204 (take a break from coding, eat some sushi!)";
        pub fn from_bytes(v: Vec<u8>) -> Self {   
            AesMsg {
                data: v,
                aad: Self::AAD.as_bytes().to_vec(),
                nonce: None,
                is_encrypted: false,
            }
        }

        pub fn from_str(s: &str) -> Self {
            let plaintext = s.as_bytes().to_vec();
   
            AesMsg {
                data: plaintext,
                aad: Self::AAD.as_bytes().to_vec(),
                nonce: None,
                is_encrypted: false,
            }
        }

        pub fn extract_str(&self) -> Result<String> {
            if self.is_encrypted {
                return Err(anyhow!("Error, message is still encrypted"));
            }

            let s = self.data.clone();
            String::from_utf8(s)
                .map_err(|e| anyhow!("data is not in utf-8 -> {}", e))
        }

    }
    
    pub struct AesEncryption {
        key: [u8; 32]
    }
    
    impl AesEncryption {
        pub fn new() -> Self {
            let rng = SystemRandom::new();
            let mut key = [0_u8; 32];
            rng.fill(&mut key)
                .unwrap();
            
            AesEncryption{
                key: key
            }
        }

        pub fn from (key: &[u8; 32]) -> Self {
            AesEncryption{
                key: key.clone()
            }
        }
        
        pub fn from_tyb_apikey(apikey: impl AsRef<[u8]>) -> Self {
            let mut key = [0_u8; 32];
            for (i, c) in hash_utils::sha256(apikey).chars().enumerate() {
                if i >= 32 {
                    break;
                }
                key[i] = c as u8;
            }

            AesEncryption {
                key: key
            }
        }

        pub fn encrypt(&self, msg: &mut AesMsg) -> Result<(), String> {
            let rng = SystemRandom::new();
    
            let mut in_out = if !msg.is_encrypted {
                msg.data.clone()
            }
            else {
                return Err("message is already encrypted".to_string());
            };
    
            // Prepare the key
            let unbound_key = match UnboundKey::new(&AES_256_GCM, &self.key) {
                Ok(k) => k,
                Err(e) => return Err(format!("Failed to create new key: {}", e)),
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
    
        pub fn decrypt(&self, msg: &mut AesMsg) -> Result<(), String> {
            // Prepare the key
            let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key).unwrap();
            let less_safe_key = LessSafeKey::new(unbound_key);
    
            let ciphertext = if msg.is_encrypted {
                msg.data.clone()
            }
            else {
                return Err("no ciphertext in message".to_string());
            };
            let nonce = if let Some(nonce) = msg.nonce.clone() {
                msg.nonce = None;
                nonce
            }
            else {
                return Err("no nonce in message".to_string());
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
}