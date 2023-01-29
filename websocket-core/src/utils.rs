use std::str::{from_utf8, Utf8Error};

pub fn bytes_to_string(data: &[u8]) -> Result<String, Utf8Error> {
    let utf8 = from_utf8(data)?;
    Ok(utf8.to_string())
}
