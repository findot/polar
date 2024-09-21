use crate::result::CryptoError;
use pem::parse as parse_pem;
use rocket::tokio::{
    fs::read as read_file,
    io::{Error as IOError, ErrorKind},
};
use std::path::Path;

pub async fn extract_key(key_path_str: &str) -> Result<Vec<u8>, CryptoError> {
    let key_path = Path::new(key_path_str);
    if !key_path.exists() {
        Err(IOError::new(ErrorKind::NotFound, key_path_str))?;
    }
    if !key_path.is_file() {
        return Err(IOError::new(ErrorKind::InvalidInput, key_path_str))?;
    }
    let file_content = read_file(key_path).await?;
    let bytes = parse_pem(file_content)?;

    Ok(bytes.into_contents())
}
