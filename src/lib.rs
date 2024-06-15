pub mod docker_utils;
pub mod proj_utils;
pub mod crypt_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crypt_utils::{compression_utils, rsa_utils};
    
    #[test]
    fn encryption_and_compression() {
        let text = std::fs::read_to_string("./Cargo.lock")
            .unwrap();
        
        let mut packet = crypt_utils::BinaryPacket::from(&text)
            .unwrap();

        compression_utils::compress_brotli(&mut packet)
            .expect("error");

        let keys = crypt_utils::RsaKeys::new();

        rsa_utils::encrypt(&mut packet, &keys.pub_key)
            .unwrap();

        let s = format!("Encrypted Data: {:?}\nSize: {}", &packet, packet.mem_size());
        std::fs::write("./test-outputs/encrypted-data.txt", &s)
            .unwrap();

        rsa_utils::decrypt(&mut packet, &keys.priv_key)
            .unwrap();

        compression_utils::decompress_brotli(&mut packet)
            .unwrap();

        let text: String = bincode::deserialize(&packet.data).unwrap();
        let s = format!("\n\nUnencrypted Data: {}\nSize: {}", &text, packet.mem_size());
        std::fs::write("./test-outputs/decrypted-data.txt", &s)
        .unwrap();

        assert!(text == std::fs::read_to_string("./Cargo.lock").unwrap());

    }

    #[test]
    fn apikey_generation() {
        let key = crypt_utils::gen_apikey();
        assert!(key.starts_with("tyb_key_"));
        assert!(key.len() > 64);
    }

}