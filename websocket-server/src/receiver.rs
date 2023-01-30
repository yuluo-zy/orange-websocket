use std::io;
use std::io::{BufReader, Read};
use std::net::Shutdown;
use websocket_core::action::receiver::{DataFrameIterator, MessageIterator, Receiver as ReceiverAble};
use websocket_core::dataframe::DataFrame;
use websocket_core::error::WebSocketError;
use websocket_core::message::Message;
use websocket_core::protocol::header::Opcode;
use websocket_core::stream::{AsTcpStream, Stream};
use crate::WebSocketResult;

const DEFAULT_MAX_DATAFRAME_SIZE : usize = 1024*1024*100;
const DEFAULT_MAX_MESSAGE_SIZE : usize = 1024*1024*200;
const MAX_DATAFRAMES_IN_ONE_MESSAGE: usize = 1024*1024;
const PER_DATAFRAME_OVERHEAD : usize = 64;



pub struct Receiver {
    buffer: Vec<DataFrame>,
    mask: bool,
    max_dataframe_size: u32,
    max_message_size: u32,
}

impl Receiver {
    pub fn new(mask: bool) -> Receiver {
        Receiver::new_with_limits(mask, DEFAULT_MAX_DATAFRAME_SIZE, DEFAULT_MAX_MESSAGE_SIZE)
    }

    pub fn new_with_limits(mask: bool, max_dataframe_size: usize, max_message_size: usize) -> Receiver {
        let max_dataframe_size: u32 = max_dataframe_size.min(u32::MAX as usize) as u32;
        let max_message_size: u32 = max_message_size.min(u32::MAX as usize) as u32;
        Receiver {
            buffer: Vec::new(),
            mask,
            max_dataframe_size,
            max_message_size,
        }
    }
}

impl ReceiverAble for Receiver {
    type F = DataFrame;

    type M = Message;

    fn recv_dataframe<R>(&mut self, reader: &mut R) -> WebSocketResult<DataFrame>
        where
            R: Read,
    {
        DataFrame::read_dataframe_with_limit(reader, self.mask, self.max_dataframe_size as usize)
    }

    fn recv_message_dataframes<R>(&mut self, reader: &mut R) -> WebSocketResult<Vec<DataFrame>>
        where
            R: Read,
    {
        let mut current_message_length : usize = self.buffer.iter().map(|x|x.data.len()).sum();
        let mut finished = if self.buffer.is_empty() {
            let first = self.recv_dataframe(reader)?;

            if first.opcode == Opcode::Continuation {
                return Err(WebSocketError::ProtocolError(
                    "Unexpected continuation data frame opcode",
                ));
            }

            let finished = first.finished;
            current_message_length += first.data.len() + PER_DATAFRAME_OVERHEAD;
            self.buffer.push(first);
            finished
        } else {
            false
        };

        while !finished {
            let next = self.recv_dataframe(reader)?;
            finished = next.finished;

            match next.opcode as u8 {
                // Continuation opcode
                0 => {
                    current_message_length += next.data.len() + PER_DATAFRAME_OVERHEAD;
                    self.buffer.push(next)
                }
                // Control frame
                8..=15 => {
                    return Ok(vec![next]);
                }
                // Others
                _ => {
                    return Err(WebSocketError::ProtocolError(
                        "Unexpected data frame opcode",
                    ));
                }
            }

            if !finished {
                if self.buffer.len() >= MAX_DATAFRAMES_IN_ONE_MESSAGE {
                    return Err(WebSocketError::ProtocolError(
                        "Exceeded count of data frames in one WebSocket message",
                    ));
                }
                if current_message_length >= self.max_message_size as usize {
                    return Err(WebSocketError::ProtocolError(
                        "Exceeded maximum WebSocket message size",
                    ));
                }
            }
        }

        Ok(std::mem::replace(&mut self.buffer, Vec::new()))
    }
}

pub struct Reader<R>
    where
        R: Read,
{

    pub stream: BufReader<R>,
    pub receiver: Receiver,
}

impl<R> Reader<R> where R: Read {

    pub fn recv_dataframe(&mut self) -> WebSocketResult<DataFrame> {
        self.receiver.recv_dataframe(&mut self.stream)
    }


    pub fn incoming_dataframes(&mut self) -> DataFrameIterator<Receiver, BufReader<R>> {
        self.receiver.incoming_dataframes(&mut self.stream)
    }

    pub fn recv_message(&mut self) -> WebSocketResult<Message> {
        self.receiver.recv_message(&mut self.stream)
    }

    pub fn incoming_messages(&mut self) -> MessageIterator<Receiver, BufReader<R>> {
        self.receiver.incoming_messages(&mut self.stream)
    }
}

impl<S> Reader<S> where S: AsTcpStream + Stream + Read{
    pub fn shutdown(&self) -> io::Result<()> {
        self.stream.get_ref().as_tcp().shutdown(Shutdown::Read)
    }

    pub fn shutdown_all(&self) -> io::Result<()> {
        self.stream.get_ref().as_tcp().shutdown(Shutdown::Both)
    }
}