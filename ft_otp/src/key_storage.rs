use std::fs::File;
use std::io::{Read, Write, Error, ErrorKind};

pub struct KeyStorage {
  file_path: String,
  encryption_key: Vec<u8>,
}

impl KeyStorage {

  pub fn new(file_path: &str, encryption_key: &str) -> Self {
    KeyStorage {
      file_path: file_path.to_string(),
      encryption_key: encryption_key.as_bytes().to_vec(),
    }
  }

  /// Encrypts or decrypts data
  /// XOR is useful here because it can be used both ways, but is unsafe in real world apps
  fn xor_encrypt_decrypt(&self, data: &[u8]) -> Vec<u8> {
    data.iter()
      .zip(self.encryption_key.iter().cycle())
      .map(|(data_byte, key_byte)| data_byte ^ key_byte)
      .collect()
  }

  pub fn store_key(&self, hex_key: &str) -> Result<(), Error> {
    let key_bytes = self.hex_string_to_bytes(hex_key)?;
    let encrypted = self.xor_encrypt_decrypt(&key_bytes);
    
    let mut file = File::create(&self.file_path)?;
    file.write_all(&encrypted)
  }

  pub fn read_key(&self) -> Result<Vec<u8>, Error> {
    let mut file = File::open(&self.file_path)?;
    let mut encrypted = Vec::new();
    file.read_to_end(&mut encrypted)?;
    
    Ok(self.xor_encrypt_decrypt(&encrypted))
  }

  fn hex_string_to_bytes(&self, hex_string: &str) -> Result<Vec<u8>, Error> {
    hex_string.chars()
      .collect::<Vec<char>>()
      .chunks(2)
      .map(|chunk| {
        let hex_pair: String = chunk.iter().collect();
        u8::from_str_radix(&hex_pair, 16)
          .map_err(|e| Error::new(ErrorKind::InvalidData, e))
      })
      .collect()
  }
}