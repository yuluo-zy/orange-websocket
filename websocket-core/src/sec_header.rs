use std::fmt;
use std::fmt::Debug;
use std::str::FromStr;
use base64::Engine;
use base64::engine::general_purpose;
use sha1::{Sha1, Digest};
use crate::error::WebSocketError;
use crate::result::WebSocketResult;

pub mod names {
    pub const PROTOCOL: &str = "Sec-WebSocket-Protocol";
    pub const ACCEPT: &str = "Sec-WebSocket-Accept";
    pub const EXTENSIONS: &str = "Sec-WebSocket-Extensions";
    pub const KEY: &str = "Sec-WebSocket-Key";
}
#[derive(PartialEq, Clone, Copy, Default)]
pub struct WebSocketKey([u8; 16]);

impl FromStr for WebSocketKey {
    type Err = WebSocketError;

    fn from_str(key: &str) -> WebSocketResult<WebSocketKey> {
        match general_purpose::URL_SAFE_NO_PAD.decode(key) {
            Ok(vec) => {
                if vec.len() != 16 {
                    return Err(WebSocketError::ProtocolError(
                        "Sec-WebSocket-Key must be 16 bytes",
                    ));
                }
                let mut array = [0u8; 16];
                array[..16].clone_from_slice(&vec[..16]);
                Ok(WebSocketKey(array))
            }
            Err(_) => Err(WebSocketError::ProtocolError(
                "Invalid Sec-WebSocket-Accept",
            )),
        }
    }
}

impl WebSocketKey {
    /// Generate a new, random WebSocketKey
    pub fn new() -> WebSocketKey {
        let key = rand::random();
        WebSocketKey(key)
    }
    /// Return the Base64 encoding of this WebSocketKey
    pub fn serialize(&self) -> String {
        let WebSocketKey(key) = *self;
        general_purpose::URL_SAFE_NO_PAD.encode(&key)
    }
}

static MAGIC_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

/// Represents a Sec-WebSocket-Accept header
#[derive(PartialEq, Clone, Copy)]
pub struct WebSocketAccept([u8; 20]);

impl Debug for WebSocketAccept {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebSocketAccept({})", self.serialize())
    }
}

impl FromStr for WebSocketAccept {
    type Err = WebSocketError;

    fn from_str(accept: &str) -> WebSocketResult<WebSocketAccept> {
        match general_purpose::URL_SAFE_NO_PAD.decode(accept) {
            Ok(vec) => {
                if vec.len() != 20 {
                    return Err(WebSocketError::ProtocolError(
                        "Sec-WebSocket-Accept must be 20 bytes",
                    ));
                }
                let mut array = [0u8; 20];
                array[..20].clone_from_slice(&vec[..20]);
                Ok(WebSocketAccept(array))
            }
            Err(_) => Err(WebSocketError::ProtocolError(
                "Invalid Sec-WebSocket-Accept ",
            )),
        }
    }
}

impl WebSocketAccept {
    /// Create a new WebSocketAccept from the given WebSocketKey
    pub fn new(key: &WebSocketKey) -> WebSocketAccept {
        let serialized = key.serialize();
        let mut concat_key = String::with_capacity(serialized.len() + 36);
        concat_key.push_str(&serialized[..]);
        concat_key.push_str(MAGIC_GUID);
        let hash = Sha1::digest(concat_key.as_bytes());
        WebSocketAccept(hash.into())
    }
    /// Return the Base64 encoding of this WebSocketAccept
    pub fn serialize(&self) -> String {
        let WebSocketAccept(accept) = *self;
        general_purpose::URL_SAFE_NO_PAD.encode(&accept)
    }
}
