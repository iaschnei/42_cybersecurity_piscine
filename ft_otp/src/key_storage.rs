use std::fs::File;
use std::io::{Read, Write, Error};

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

  // Encrypt the data using xor of data / encrypt key
  // Useful because you only have to run it again to decrypt but not safe for real world app
  pub fn xor_encrypt_decrypt(&self, data: &[u8]) -> Vec<u8> {
    data.iter()
      .zip(self.encryption_key.iter().cycle())
      .map(|(a, b)| a ^ b)
      .collect()
  }

  pub fn store_key(&self, hexa_key: &str) -> Result<(), Error>{
    let mut file = File::create(&self.file_path)?;
    let encrypted = self.xor_encrypt_decrypt(hexa_key.as_bytes());
    file.write_all(&encrypted)?;
    Ok(())
  }

  pub fn read_key(&self) -> Result<String, Error> {
    let mut file = File::open(&self.file_path)?;
    let mut encrypted = Vec::new();
    file.read_to_end(&mut encrypted)?;
    let decrypted = self.xor_encrypt_decrypt(&encrypted);
    String::from_utf8(decrypted)
      .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))
  }
}

// new:     Create a new storage (file) and associate a key with it : 4 digit key as arg
// store:   Encrypt the given key using XOR with the storage key and write it in file
// read:    Decrypt using the same XOR and return the new string