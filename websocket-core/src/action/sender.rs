use std::io::Write;
use crate::protocol::dataframe::DataFrame;
use crate::protocol::message::Message;
use crate::result::WebSocketResult;

pub trait Sender {

    fn is_masked(&self) -> bool;

    fn send_dataframe<D, W>(&mut self, writer: &mut W, dataframe: &D) -> WebSocketResult<()>
        where
            D: DataFrame,
            W: Write,
    {
        dataframe.write_to(writer, self.is_masked())?;
        Ok(())
    }

    /// Sends a single message using this sender.
    fn send_message<M, W>(&mut self, writer: &mut W, message: &M) -> WebSocketResult<()>
        where
            M: Message,
            W: Write,
    {
        message.serialize(writer, self.is_masked())?;
        Ok(())
    }
}