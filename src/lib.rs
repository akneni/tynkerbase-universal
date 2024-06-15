mod docker_utils;
mod proj_utils;
mod crypt_utils;


#[cfg(test)]
mod tests {
    use super::*;
    use crypt_utils::{compression_utils, rsa_utils};
    
    #[test]
    fn test_encryption_and_compression() {
        let v = std::fs::read_to_string("./Cargo.lock")
            .unwrap()
            .as_bytes()
            .to_vec();
        
        let mut packet = crypt_utils::BinaryPacket::from_bytes(v);

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

        let text = String::from_utf8(packet.data.clone()).unwrap();
        let s = format!("\n\nUnencrypted Data: {}\nSize: {}", &text, packet.mem_size());
        std::fs::write("./test-outputs/decrypted-data.txt", &s)
        .unwrap();

        assert!(text == std::fs::read_to_string("./Cargo.lock").unwrap());

    }
}