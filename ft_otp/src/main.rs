use std::{fs::File, io::Read, path::Path, time::SystemTime};
use clap::{Parser, ArgGroup};

mod key_storage;
use key_storage::KeyStorage;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about)]
#[command(group(ArgGroup::new("output_path").required(true).args(["generate_path", "key_path"])))]
struct Args {
  /// The encrypt key for the file
  #[arg(short = 'x', long, value_parser = expect_four_digits)]
  x: String,

  /// Generate a new key file with the -x key
  #[arg(short = 'g', long, value_name = "PATH")]
  generate_path: Option<String>,

  /// Generate a new temp code from a key file
  #[arg(short = 'k', long, value_name = "PATH")]
  key_path: Option<String>,
}

const EXPIRE_TIME: u64 = 30;
const DIGITS: i32 = 6;

fn main() -> Result<(), std::io::Error>{

  let args = Args::parse();

  if !args.generate_path.is_none() {

    let mut hexa_key_file = File::open(&args.generate_path.clone().unwrap())?;
    let mut hexa_key = Vec::new();
    hexa_key_file.read_to_end(&mut hexa_key)?;
    if hexa_key.len() < 64 || hexa_key.len() > 160 {
      eprintln!("The key must be between 64 and 160 chars long.");
      return Ok(());
    }

    let hexa_key_str = String::from_utf8_lossy(&hexa_key).into_owned();
    if !hexa_key_str.chars().all(|c| c.is_ascii_hexdigit()) {
      eprintln!("The key can only contain hexadecimal digits");
      return Ok(());
    }

    let storage_path = Path::new(&args.generate_path.unwrap())
      .with_extension("key")
      .to_string_lossy()
      .into_owned();
    let storage = KeyStorage::new(&storage_path, &args.x);
    storage.store_key(&hexa_key_str)?;
    println!("Key saved successfully.");
    return Ok(())
  }
  else {

    let storage = KeyStorage::new(&args.key_path.unwrap(), &args.x);
    let key = storage.read_key().unwrap().to_string();
    
    //"Counter" for HOTP is replaced by a time frame in TOTP protocol
    let current_time = SystemTime::now().elapsed().expect("Couldn't get current time.").as_secs();
    let time = current_time / EXPIRE_TIME;

    let code = generate_hotp(key.into(), time, DIGITS);

    println!("{}", code);
    return Ok(());
  }
}

// All of this is in RFC 4226, so no surprise here
fn generate_hotp(secret_key: Vec<u8>, time: u64, digit: i32) -> String {

  //Generate a counter
  let mut bytes = [0u8; 8];

  let mut counter = time;
  for i in (0..8).rev() {
    bytes[i] = (counter & 0xFF) as u8;
    counter >>= 8;
  }

  let mut hmac = sha1_smol::Sha1::from(secret_key);
  hmac.update(&bytes);
  let res = hmac.digest().bytes();

  let offset = (res.last().unwrap() & 0x0F) as usize;

  let truncated = ((res[offset] & 0x7F) as i32) << 24 |
  ((res[offset + 1] & 0xFF) as i32) << 16 |
  ((res[offset + 2] & 0xFF) as i32) << 8 |
  (res[offset + 3] & 0xFF) as i32;

  let code = (truncated % 10 ^ digit).to_string();
  
  code
}

fn expect_four_digits(input: &str) -> Result<String, String> {
  if input.len() != 4 || !input.chars().all(|c| c.is_ascii_digit()) {
    return Err("Value must be exactly 4 digits".to_string())
  }
  Ok(input.to_string())
}