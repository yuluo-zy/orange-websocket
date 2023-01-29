use std::io::{Read, Write};
use bitflags::bitflags;
use crate::codec::order_byte::{NetworkEndian, ReadBytesExt, WriteBytesExt};
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
    fn read(reader: &mut impl Read) -> WebSocketResult<Self>;
    fn write(self, writer: &mut impl Write) -> WebSocketResult<()>;
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
        if self.opcode > 0xF {
            return Err(WebSocketError::DataFrameError("Invalid data frame opcode"));
        }
        if self.opcode >= 8 && self.len >= 126 {
            return Err(WebSocketError::DataFrameError(
                "Control frame length too long",
            ));
        }

        // Write 'FIN', 'RSV1', 'RSV2', 'RSV3' and 'opcode'
        writer.write_u8((self.flags.bits) | self.opcode)?;

        writer.write_u8(
            // Write the 'MASK'
            if self.mask.is_some() { 0x80 } else { 0x00 } |
                // Write the 'Payload len'
                if self.len <= 125 { self.len as u8 }
                else if self.len <= 65535 { 126 }
                else { 127 },
        )?;

        // Write 'Extended payload length'
        if self.len >= 126 && self.len <= 65535 {
            writer.write_u16::<NetworkEndian>(self.len as u16)?;
        } else if self.len > 65535 {
            writer.write_u64::<NetworkEndian>(self.len)?;
        }

        // Write 'Masking-key'
        if let Some(mask) = self.mask {
            writer.write_all(&mask)?
        }

        Ok(())
    }
}

pub struct DataMasker<'w, T> where T : Write{
    key: [u8; 4],
    pos: usize,
    endpoint: &'w mut T
}

impl<'w, T> DataMasker<'w, T> where T: Write {
    pub fn new(key: [u8; 4], endpoint: &'w mut T) -> Self {
        Self {
            key,
            pos: 0,
            endpoint
        }
    }
}

impl<'w, T> Write for DataMasker<'w, T> where T: Write {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let mut data = Vec::with_capacity(buf.len());
            for &byte in buf.iter() {
                data.push(byte ^ self.key[self.pos]);
                self.pos = (self.pos + 1) % self.key.len();
            }
            self.endpoint.write(&data)
    }

    fn flush(&mut self) -> std::io::Result<()> {
       self.endpoint.flush()
    }
}

pub fn gen_mask() -> [u8; 4] {
    rand::random()
}

pub fn mask_data(mask: [u8; 4], data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let zip_iter = data.iter().zip(mask.iter().cycle());
    for (&buf_item, &key_item) in zip_iter {
        out.push(buf_item ^ key_item);
    }
    out
}
