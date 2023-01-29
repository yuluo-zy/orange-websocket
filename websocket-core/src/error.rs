use std::io;
use std::str::Utf8Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("WebSocket data frame error {0}")]
    DataFrameError(&'static str),
    #[error("WebSocket protocol error {0}")]
    ProtocolError(&'static str),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error)
}