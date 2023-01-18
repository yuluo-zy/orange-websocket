use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WebSocketError {
    #[error("WebSocket data frame error {0}")]
    DataFrameError(&'static str),
    #[error("WebSocket protocol error {0}")]
    ProtocolError(&'static str),
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
}