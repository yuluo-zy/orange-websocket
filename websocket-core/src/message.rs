use std::io;
use std::io::Write;
use std::str::from_utf8;
use crate::codec::order_byte::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use crate::error::WebSocketError;
use crate::protocol;
use crate::protocol::dataframe::DataFrame;
use crate::protocol::header::Opcode;
use crate::protocol::message::Type;
use crate::result::WebSocketResult;
use crate::utils::bytes_to_string;

const FALSE_RESERVED_BITS: &[bool; 3] = &[false; 3];

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Message {
    /// Type of WebSocket message
    pub opcode: Type,
    /// Optional status code to send when closing a connection.
    /// (only used if this message is of Type::Close)
    pub cd_status_code: Option<u16>,
    /// Main payload
    pub payload: Vec<u8>,
}

impl Message {
    fn new(code: Type, status: Option<u16>, payload: Vec<u8>) -> Self {
        Message {
            opcode: code,
            cd_status_code: status,
            payload
        }
    }

    pub fn text(data: String) -> Self
    {
        Message::new(
            Type::Text,
            None,
            data.into_bytes(),
        )
    }

    pub fn binary(data: Vec<u8>) -> Self
    {
        Message::new(Type::Binary, None, data)
    }

    pub fn close() -> Self {
        Message::new(Type::Close, None, vec![0])
    }

    pub fn close_because(code: u16, reason: String) -> Self

    {
        Message::new(
			Type::Close,
			Some(code),
			reason.into_bytes(),
		)
    }

    pub fn ping(data: Vec<u8>) -> Self
    {
        Message::new(Type::Ping, None, data)
    }

    pub fn pong(data: Vec<u8>) -> Self
    {
        Message::new(Type::Pong, None, data)
    }

    pub fn to_pong(&mut self) -> Result<(), ()> {
        if self.opcode == Type::Ping {
            self.opcode = Type::Pong;
            Ok(())
        } else {
            Err(())
        }
    }
}

impl DataFrame for Message {
    #[inline(always)]
    fn is_last(&self) -> bool {
        true
    }

    #[inline(always)]
    fn opcode(&self) -> u8 {
        self.opcode as u8
    }

    #[inline(always)]
    fn reserved(&self) -> &[bool; 3] {
        FALSE_RESERVED_BITS
    }

    fn size(&self) -> usize {
        self.payload.len() + if self.cd_status_code.is_some() { 2 } else { 0 }
    }

    fn write_payload(&self, socket: &mut impl Write) -> WebSocketResult<()> {
        if let Some(reason) = self.cd_status_code {
            socket.write_u16::<NetworkEndian>(reason)?;
        }
        socket.write_all(&*self.payload)?;
        Ok(())
    }

    fn take_payload(self) -> Vec<u8> {
        if let Some(reason) = self.cd_status_code {
            let mut buf = Vec::with_capacity(2 + self.payload.len());
            buf.write_u16::<NetworkEndian>(reason)
                .expect("failed to write close code in take_payload");
            buf.append(&mut self.payload.to_vec());
            buf
        } else {
            self.payload.to_vec()
        }
    }
}

impl protocol::message::Message for Message {
    /// Attempt to form a message from a series of data frames
    fn serialize(&self, writer: &mut impl Write, masked: bool) -> WebSocketResult<()> {
        self.write_to(writer, masked)
    }

    /// Returns how many bytes this message will take up
    fn message_size(&self, masked: bool) -> usize {
        self.frame_size(masked)
    }

    /// Attempt to form a message from a series of data frames
    fn from_dataframes<D>(frames: Vec<D>) -> WebSocketResult<Self>
        where
            D: DataFrame,
    {
        let opcode = frames
            .first()
            .ok_or(WebSocketError::ProtocolError("No dataframes provided"))
            .map(DataFrame::opcode)?;
        let opcode = Opcode::new(opcode);

        let payload_size = frames.iter().map(DataFrame::size).sum();

        let mut data = Vec::with_capacity(payload_size);

        for (i, dataframe) in frames.into_iter().enumerate() {
            if i > 0 && dataframe.opcode() != Opcode::Continuation as u8 {
                return Err(WebSocketError::ProtocolError(
                    "Unexpected non-continuation data frame",
                ));
            }
            if *dataframe.reserved() != [false; 3] {
                return Err(WebSocketError::ProtocolError(
                    "Unsupported reserved bits received",
                ));
            }
            data.append(&mut dataframe.take_payload());
        }

        if opcode == Some(Opcode::Text) {
            if let Err(e) = from_utf8(data.as_slice()) {
                return Err(e.into());
            }
        }

        let msg = match opcode {
            Some(Opcode::Text) => Message {
                opcode: Type::Text,
                cd_status_code: None,
                payload: data,
            },
            Some(Opcode::Binary) => Message::binary(data),
            Some(Opcode::Close) => {
                if !data.is_empty() {
                    let status_code = (&data[..]).read_u16::<NetworkEndian>()?;
                    let reason = bytes_to_string(&data[2..])?;
                    Message::close_because(status_code, reason)
                } else {
                    Message::close()
                }
            }
            Some(Opcode::Ping) => Message::ping(data),
            Some(Opcode::Pong) => Message::pong(data),
            _ => return Err(WebSocketError::ProtocolError("Unsupported opcode received")),
        };
        Ok(msg)
    }
}

/// Represents data contained in a Close message
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct CloseData {
    /// The status-code of the CloseData
    pub status_code: u16,
    /// The reason-phrase of the CloseData
    pub reason: String,
}

impl CloseData {
    /// Create a new CloseData object
    pub fn new(status_code: u16, reason: String) -> CloseData {
        CloseData {
            status_code,
            reason,
        }
    }
    /// Convert this into a vector of bytes
    pub fn into_bytes(self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.write_u16::<NetworkEndian>(self.status_code)?;
        for i in self.reason.as_bytes().iter() {
            buf.push(*i);
        }
        Ok(buf)
    }
}
