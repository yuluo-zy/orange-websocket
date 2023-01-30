use websocket_core::error::WebSocketError;

mod header;
mod error;
mod receiver;
mod sender;

pub type WebSocketResult<T> = Result<T, WebSocketError>;