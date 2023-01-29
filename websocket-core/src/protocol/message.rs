use std::io::Write;
use crate::protocol::dataframe::DataFrame;
use crate::result::WebSocketResult;

/// Valid types of messages (in the default implementation)
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Type {
    /// Message with UTF8 test
    Text = 1,
    /// Message containing binary data
    Binary = 2,
    /// Ping message with data
    Ping = 9,
    /// Pong message with data
    Pong = 10,
    /// Close connection message with optional reason
    Close = 8,
}

pub trait Message: Sized {
    /// Writes this message to the writer
    fn serialize(&self, _: &mut impl Write, masked: bool) -> WebSocketResult<()>;

    /// Returns how many bytes this message will take up
    fn message_size(&self, masked: bool) -> usize;

    /// Attempt to form a message from a series of data frames
    fn from_dataframes<D: DataFrame>(frames: Vec<D>) -> WebSocketResult<Self>;
}