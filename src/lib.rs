pub mod file_utils;
pub mod crypt_utils;
pub mod netwk_utils;
pub mod constants;

#[cfg(test)]
mod tests {
    use super::*;
    use crypt_utils::{compression_utils, aes_utils};
    use std::{
        path::Path,
        fs,
    };

    
    #[test]
    fn compression() {
        let text = std::fs::read_to_string("./Cargo.lock")
            .unwrap();
        
        let mut packet = crypt_utils::BinaryPacket::from(&text)
            .unwrap();

        compression_utils::compress_brotli(&mut packet)
            .expect("error");

        let s = format!("Compressed Data: {:?}\nSize: {}", &packet, packet.data.len());
        if !Path::new("./test-outputs").exists() {
            fs::create_dir("./test-outputs").unwrap();
        }
        fs::write("./test-outputs/encrypted-data.txt", &s)
            .unwrap();

        compression_utils::decompress_brotli(&mut packet)
            .unwrap();

        let text: String = bincode::deserialize(&packet.data).unwrap();
        let s = format!("\n\nUnencrypted Data: {}\nSize: {}", &text, packet.data.len());
        std::fs::write("./test-outputs/decrypted-data.txt", &s)
        .unwrap();

        assert!(text == std::fs::read_to_string("./Cargo.lock").unwrap());

    }

    #[test]
    fn apikey_generation() {
        let key = crypt_utils::gen_apikey("keys", "salt");
        if !Path::new("./test-outputs").exists() {
            fs::create_dir("./test-outputs").unwrap();
        }
        std::fs::write("./test-outputs/out.txt", &key).unwrap();
        assert!(key.starts_with("tyb_key_"));
        assert!(key.len() > 64);
    }

    #[test]
    fn aes_keygen() {
        let key = crypt_utils::gen_apikey("keys", "salt");
        let aes = aes_utils::AesEncryption::from_tyb_apikey(&key);

        let plaintext = "hey!!";
        let mut msg = aes_utils::AesMsg::from_str(&plaintext);

        aes.encrypt(&mut msg)
            .unwrap();

        let mut msg2 = msg.clone();

        aes.decrypt(&mut msg2).unwrap();

        let res: String = msg2.extract_str().unwrap();

        assert_eq!(res, plaintext);       
    }

}