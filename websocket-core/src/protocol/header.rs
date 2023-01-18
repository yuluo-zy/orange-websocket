use std::io::{Read, Write};
use bitflags::bitflags;
use crate::codec::order_byte::{NetworkEndian, ReadBytesExt};
use crate::error::WebSocketError;
use crate::result::WebSocketResult;

bitflags! {
	/// Flags relevant to a WebSocket data frame.
	pub struct DataFrameFlags: u8 {
		/// Marks this dataframe as the last dataframe
		const FIN = 0x80;
		/// First reserved bit
		const RSV1 = 0x40;
		/// Second reserved bit
		const RSV2 = 0x20;
		/// Third reserved bit
		const RSV3 = 0x10;
	}
}

pub trait FrameHeader: Sized {
    fn read(reader: &mut dyn Read) -> WebSocketResult<Self>;
    fn write(self, writer: &mut dyn Write) -> WebSocketResult<()>;
}

pub struct DataFrameHeader {
    /// The bit flags for the first byte of the header.
    pub flags: DataFrameFlags,
    /// The opcode of the header - must be <= 16.
    pub opcode: u8,
    /// The masking key, if any.
    pub mask: Option<[u8; 4]>,
    /// The length of the payload.
    pub len: u64,
}

impl FrameHeader for DataFrameHeader {
    fn read(reader: &mut impl Read) -> WebSocketResult<Self> {
        let byte0 = reader.read_u8()?;
        let byte1 = reader.read_u8()?;

        let flags = DataFrameFlags::from_bits_truncate(byte0);
        let opcode = byte0 & 0x0F;

        let len = match byte1 & 0x7F {
            0..=125 => u64::from(byte1 & 0x7F),
            126 => {
                let len = u64::from(reader.read_u16::<NetworkEndian>()?);
                if len <= 125 {
                    return Err(WebSocketError::DataFrameError("Invalid data frame length"));
                }
                len
            }
            127 => {
                let len = reader.read_u64::<NetworkEndian>()?;
                if len <= 65535 {
                    return Err(WebSocketError::DataFrameError("Invalid data frame length"));
                }
                len
            }
            _ => unreachable!(),
        };

        if opcode >= 8 {
            if len >= 126 {
                return Err(WebSocketError::DataFrameError(
                    "Control frame length too long",
                ));
            }
            if !flags.contains(DataFrameFlags::FIN) {
                return Err(WebSocketError::ProtocolError(
                    "Illegal fragmented control frame",
                ));
            }
        }

        let mask = if byte1 & 0x80 == 0x80 {
            Some([
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
                reader.read_u8()?,
            ])
        } else {
            None
        };

        Ok(DataFrameHeader {
            flags,
            opcode,
            mask,
            len,
        })
    }

    fn write(self, writer: &mut impl Write) -> WebSocketResult<()> {
        Ok(())
    }
}
