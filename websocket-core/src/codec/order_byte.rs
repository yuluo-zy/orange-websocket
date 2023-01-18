use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::Error;

pub type IoError<T> = Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NetworkEndian {}

pub trait ByteOrder:
Clone
+ Copy
+ Debug
+ Default
+ Eq
+ Hash
+ Ord
+ PartialEq
+ PartialOrd {
    fn read_u16(buf: &[u8]) -> u16;
    fn read_u32(buf: &[u8]) -> u32;
    fn read_u64(buf: &[u8]) -> u64;
    fn write_u16(buf: &mut [u8], n: u16);
    fn write_u32(buf: &mut [u8], n: u32);
    fn write_u64(buf: &mut [u8], n: u64);
}

impl Default for NetworkEndian {
    fn default() -> Self {
        panic!("NetWorkEndian default")
    }
}

impl ByteOrder for NetworkEndian {
    #[inline]
    fn read_u16(buf: &[u8]) -> u16 {
        u16::from_be_bytes(buf[..2].try_into().unwrap())
    }

    #[inline]
    fn read_u32(buf: &[u8]) -> u32 {
        u32::from_be_bytes(buf[..4].try_into().unwrap())
    }

    #[inline]
    fn read_u64(buf: &[u8]) -> u64 {
        u64::from_be_bytes(buf[..8].try_into().unwrap())
    }

    fn write_u16(buf: &mut [u8], n: u16) {
        todo!()
    }

    fn write_u32(buf: &mut [u8], n: u32) {
        todo!()
    }

    fn write_u64(buf: &mut [u8], n: u64) {
        todo!()
    }
}

pub trait ReadBytesExt: io::Read {
    #[inline]
    fn read_u8(&mut self) -> IoError<u8> {
        let mut buf = [0; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }
    #[inline]
    fn read_u16<T: ByteOrder>(&mut self) -> IoError<u16> {
        let mut buf = [0; 2];
        self.read_exact(&mut buf)?;
        Ok(T::read_u16(&buf))
    }
    #[inline]
    fn read_u32<T: ByteOrder>(&mut self) -> IoError<u32> {
        let mut buf = [0; 4];
        self.read_exact(&mut buf)?;
        Ok(T::read_u32(&buf))
    }
    #[inline]
    fn read_u64<T: ByteOrder>(&mut self) -> IoError<u64> {
        let mut buf = [0; 8];
        self.read_exact(&mut buf)?;
        Ok(T::read_u64(&buf))
    }
}

impl<R: io::Read + ?Sized> ReadBytesExt for R {}