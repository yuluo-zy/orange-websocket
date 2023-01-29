use std::fmt::Debug;
use std::hash::Hash;
use std::io;
use std::io::Error;
use std::ptr::copy_nonoverlapping;

/// Copies $size bytes from a number $n to a &mut [u8] $dst. $ty represents the
/// numeric type of $n and $which must be either to_be or to_le, depending on
/// which endianness one wants to use when writing to $dst.
///
/// This macro is only safe to call when $ty is a numeric type and $size ==
/// size_of::<$ty>() and where $dst is a &mut [u8].
macro_rules! unsafe_write_num_bytes {
    ($ty:ty, $size:expr, $n:expr, $dst:expr, $which:ident) => {{
        assert!($size <= $dst.len());
        unsafe {
            // N.B. https://github.com/rust-lang/rust/issues/22776
            let bytes = *(&$n.$which() as *const _ as *const [u8; $size]);
            copy_nonoverlapping((&bytes).as_ptr(), $dst.as_mut_ptr(), $size);
        }
    }};
}

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

    #[inline]
    fn write_u16(buf: &mut [u8], n: u16) {
        unsafe_write_num_bytes!(u16, 2, n, buf, to_be);
    }

    #[inline]
    fn write_u32(buf: &mut [u8], n: u32) {
        unsafe_write_num_bytes!(u32, 4, n, buf, to_be);
    }

    #[inline]
    fn write_u64(buf: &mut [u8], n: u64) {
        unsafe_write_num_bytes!(u64, 8, n, buf, to_be);
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

pub trait WriteBytesExt: io::Write {

    #[inline]
    fn write_u8(&mut self, n: u8) -> IoError<()> {
        self.write_all(&[n])
    }

    #[inline]
    fn write_u16<T: ByteOrder>(&mut self, n: u16) -> IoError<()> {
        let mut buf = [0; 2];
        T::write_u16(&mut buf, n);
        self.write_all(&buf)
    }

    #[inline]
    fn write_u32<T: ByteOrder>(&mut self, n: u32) -> IoError<()> {
        let mut buf = [0; 4];
        T::write_u32(&mut buf, n);
        self.write_all(&buf)
    }

    #[inline]
    fn write_u64<T: ByteOrder>(&mut self, n: u64) -> IoError<()> {
        let mut buf = [0; 8];
        T::write_u64(&mut buf, n);
        self.write_all(&buf)
    }
}

impl<W: io::Write + ?Sized> WriteBytesExt for W {}


