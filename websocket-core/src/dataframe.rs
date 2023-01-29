//! Module containing the default implementation of data frames.
use crate::error::{WebSocketError};
use std::io::{self, Read, Write};
use crate::protocol::dataframe::DataFrame as DataFrameAble;
use crate::protocol::header::{DataFrameFlags, DataFrameHeader, FrameHeader, mask_data, Opcode};
use crate::result::WebSocketResult;

#[derive(Debug, Clone, PartialEq)]
pub struct DataFrame {
    /// Whether or no this constitutes the end of a message
    pub finished: bool,
    /// The reserved portion of the data frame (RFC6455 5.2)
    pub reserved: [bool; 3],
    /// The opcode associated with this data frame
    pub opcode: Opcode,
    /// The payload associated with this data frame
    pub data: Vec<u8>,
}

impl DataFrame {
    /// Creates a new DataFrame.
    pub fn new(finished: bool, opcode: Opcode, data: Vec<u8>) -> DataFrame {
        DataFrame {
            finished,
            reserved: [false; 3],
            opcode,
            data,
        }
    }

    /// Take the body and header of a dataframe and combine it into a single
    /// Dataframe struct. A websocket message can be made up of many individual
    /// dataframes, use the methods from the Message or OwnedMessage structs to
    /// take many of these and create a websocket message.
    pub fn read_dataframe_body(
        header: DataFrameHeader,
        body: Vec<u8>,
        should_be_masked: bool,
    ) -> WebSocketResult<Self> {
        let finished = header.flags.contains(DataFrameFlags::FIN);

        let reserved = [
            header.flags.contains(DataFrameFlags::RSV1),
            header.flags.contains(DataFrameFlags::RSV2),
            header.flags.contains(DataFrameFlags::RSV3),
        ];

        let opcode = Opcode::new(header.opcode).expect("Invalid header opcode!");

        let data = match header.mask {
            Some(mask) => {
                if !should_be_masked {
                    return Err(WebSocketError::DataFrameError(
                        "Expected unmasked data frame",
                    ));
                }
                mask_data(mask, &body)
            }
            None => {
                if should_be_masked {
                    return Err(WebSocketError::DataFrameError("Expected masked data frame"));
                }
                body
            }
        };

        Ok(DataFrame {
            finished,
            reserved,
            opcode,
            data,
        })
    }

    /// Reads a DataFrame from a Reader.
    pub fn read_dataframe<R>(reader: &mut R, should_be_masked: bool) -> WebSocketResult<Self>
        where
            R: Read,
    {
        let header =DataFrameHeader::read(reader)?;

        let mut data: Vec<u8> = Vec::with_capacity(header.len as usize);
        let read = reader.take(header.len).read_to_end(&mut data)?;
        if (read as u64) < header.len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete payload").into());
        }

        DataFrame::read_dataframe_body(header, data, should_be_masked)
    }

    /// Reads a DataFrame from a Reader, or error out if header declares exceeding limit you specify
    pub fn read_dataframe_with_limit<R>(reader: &mut R, should_be_masked: bool, limit: usize) -> WebSocketResult<Self>
        where
            R: Read,
    {
        let header = DataFrameHeader::read(reader)?;

        if header.len > limit as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "exceeded DataFrame length limit").into());
        }
        let mut data: Vec<u8> = Vec::with_capacity(header.len as usize);
        let read = reader.take(header.len).read_to_end(&mut data)?;
        if (read as u64) < header.len {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "incomplete payload").into());
        }

        DataFrame::read_dataframe_body(header, data, should_be_masked)
    }
}

impl DataFrameAble for DataFrame {
    #[inline(always)]
    fn is_last(&self) -> bool {
        self.finished
    }

    #[inline(always)]
    fn opcode(&self) -> u8 {
        self.opcode as u8
    }

    #[inline(always)]
    fn reserved(&self) -> &[bool; 3] {
        &self.reserved
    }

    #[inline(always)]
    fn size(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    fn write_payload(&self, socket: &mut impl Write) -> WebSocketResult<()> {
        socket.write_all(self.data.as_slice())?;
        Ok(())
    }

    #[inline(always)]
    fn take_payload(self) -> Vec<u8> {
        self.data
    }
}


