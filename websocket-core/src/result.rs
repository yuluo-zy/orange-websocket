use crate::error::WebSocketError;

pub type WebSocketResult<T> = Result<T, WebSocketError>;
