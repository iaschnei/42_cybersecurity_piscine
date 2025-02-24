use std::{fs::File, io::Read, path::Path, time::SystemTime};
use clap::{Parser, ArgGroup};
use hmac::{Hmac, Mac};
use sha1::Sha1;

mod key_storage;
use key_storage::KeyStorage;

const TOTP_PERIOD: u64 = 30;       // Time step in seconds
const TOTP_DIGITS: usize = 6;      // Number of digits in the code
const TOTP_MODULUS: i32 = 1000000; // 10^TOTP_DIGITS

const MIN_HEX_KEY_LENGTH: usize = 64;
const MAX_HEX_KEY_LENGTH: usize = 160;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about = "ft_otp")]
#[command(group(
    ArgGroup::new("output_path")
        .required(true)
        .args(["generate_path", "key_path"])
))]
struct Args {
    /// The encryption key (must be exactly 4 digits)
    #[arg(short = 'x', long, value_parser = validate_encryption_key)]
    x: String,

    /// Path to generate a new key file using the encryption key
    #[arg(short = 'g', long, value_name = "PATH")]
    generate_path: Option<String>,

    /// Path to an existing key file to generate a TOTP code
    #[arg(short = 'k', long, value_name = "PATH")]
    key_path: Option<String>,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    match args.generate_path {
        Some(path) => generate_new_key(&path, &args.x),
        _none => generate_totp_code(&args.key_path.unwrap(), &args.x),
    }
}

fn generate_new_key(input_path: &str, encryption_key: &str) -> Result<(), std::io::Error> {
    let hex_key = read_hex_key(input_path)?;
    validate_hex_key(&hex_key)?;

    let output_path = Path::new(input_path)
        .with_extension("key")
        .to_string_lossy()
        .into_owned();
    
    let storage = KeyStorage::new(&output_path, encryption_key);
    storage.store_key(&hex_key)?;
    println!("Key saved successfully.");
    Ok(())
}

fn generate_totp_code(key_path: &str, encryption_key: &str) -> Result<(), std::io::Error> {
    let storage = KeyStorage::new(key_path, encryption_key);
    let key = storage.read_key()?;

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("System time before Unix epoch")
        .as_secs();
    
    let time_counter = current_time / TOTP_PERIOD;
    let code = generate_hotp(key, time_counter, TOTP_MODULUS);
    
    println!("{}", code);
    Ok(())
}

/// Follows RFC 4226's requirements
fn generate_hotp(secret_key: Vec<u8>, counter: u64, modulus: i32) -> String {
    // Convert counter to big-endian byte array
    let counter_bytes = counter.to_be_bytes();

    // Generate HMAC-SHA1
    let mut mac = Hmac::<Sha1>::new_from_slice(&secret_key)
        .expect("HMAC can take key of any size");
    mac.update(&counter_bytes);
    let hmac_result = mac.finalize().into_bytes();

    // Dynamic truncation
    let offset = (hmac_result[19] & 0x0F) as usize;
    let truncated = ((hmac_result[offset] & 0x7F) as i32) << 24 |
                   ((hmac_result[offset + 1] & 0xFF) as i32) << 16 |
                   ((hmac_result[offset + 2] & 0xFF) as i32) << 8 |
                   (hmac_result[offset + 3] & 0xFF) as i32;

    // Generate fixed-length code
    format!("{:0width$}", truncated % modulus, width = TOTP_DIGITS)
}

fn read_hex_key(path: &str) -> Result<String, std::io::Error> {
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(String::from_utf8_lossy(&content).into_owned())
}

fn validate_hex_key(key: &str) -> Result<(), std::io::Error> {
    use std::io::{Error, ErrorKind};

    if key.len() < MIN_HEX_KEY_LENGTH || key.len() > MAX_HEX_KEY_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("Key length must be between {} and {} characters", 
                   MIN_HEX_KEY_LENGTH, MAX_HEX_KEY_LENGTH)
        ));
    }

    if !key.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "Key must contain only hexadecimal digits"
        ));
    }

    Ok(())
}

fn validate_encryption_key(input: &str) -> Result<String, String> {
    if input.len() != 4 || !input.chars().all(|c| c.is_ascii_digit()) {
        return Err("Encryption key must be exactly 4 digits".to_string());
    }
    Ok(input.to_string())
}