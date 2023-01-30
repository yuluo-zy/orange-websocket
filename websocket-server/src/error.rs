use std::io;
use thiserror::Error;
use hyper::{Error as HttpError, StatusCode};

#[derive(Error, Debug)]
pub enum WsUrlError {
    #[error(" Fragments are not valid in a WebSocket URL")]
    CannotSetFragment,
    #[error(" The scheme provided is invalid for a WebSocket")]
    InvalidScheme,
    #[error(" There is no hostname or IP address to connect to")]
    NoHostName,
}

#[derive(Error, Debug)]
pub enum WebSocketOtherError {
    #[error(" WebSocket protocol error: {0}")]
    ProtocolError(&'static str),
    #[error(" Invalid WebSocket request error: {0}")]
    RequestError(&'static str),
    #[error(" Invalid WebSocket response error: {0}")]
    ResponseError(&'static str),
    #[error(" Received unexpected status code: {0}")]
    StatusCodeError(#[from] StatusCode),
    #[error(" An HTTP parsing error: {0}")]
    HttpError(#[from] HttpError),
    // #[error(" A URL parsing error: {0}")]
    // UrlError(ParseError),
    #[error(" An input/output error: {0}")]
    IoError(#[from] io::Error),
    #[error(" A WebSocket URL error: {0}")]
    WebSocketUrlError(#[from] WsUrlError),
}