use sha1::Sha1;
use sha2::{Sha256, Digest};
use std::path::Path;

pub fn sha1_file(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn verify_sha1(path: &Path, expected: &str) -> bool {
    sha1_file(path).map(|h| h == expected).unwrap_or(false)
}
