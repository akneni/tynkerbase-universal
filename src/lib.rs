pub mod docker_utils;
pub mod proj_utils;
pub mod crypt_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crypt_utils::compression_utils;
    
    #[test]
    fn compression() {
        let text = std::fs::read_to_string("./Cargo.lock")
            .unwrap();
        
        let mut packet = crypt_utils::BinaryPacket::from(&text)
            .unwrap();

        compression_utils::compress_brotli(&mut packet)
            .expect("error");

        let s = format!("Compressed Data: {:?}\nSize: {}", &packet, packet.data.len());
        std::fs::write("./test-outputs/encrypted-data.txt", &s)
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
        let key = crypt_utils::gen_apikey();
        assert!(key.starts_with("tyb_key_"));
        assert!(key.len() > 64);
    }

}