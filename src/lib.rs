#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(not(feature = "std"))]
extern crate core as std;
#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::Vec;

pub type Result<T> = std::result::Result<T, Error>;

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    fn bytes(self) -> Bytes<Self> where Self: Sized {
        Bytes { inner: self }
    }
}

pub trait Write {
    fn flush(&mut self) -> Result<()>;
    fn write(&mut self, buf: &[u8]) -> Result<usize>;
    fn write_all(&mut self, mut buf: &[u8]) -> Result<()> {
        while !buf.is_empty() {
            let len = self.write(buf)?;
            buf = &buf[len..];
        }
        Ok(())
    }
}
impl<'a, W: Write + ?Sized> Write for &'a mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> { (**self).write(buf) }

    #[inline]
    fn flush(&mut self) -> Result<()> { (**self).flush() }
}

impl Write for Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }

    #[inline]
    fn flush(&mut self) -> Result<()> { Ok(()) }
}

pub struct Bytes<R> {
    inner: R,
}
impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8>;

    fn next(&mut self) -> Option<Result<u8>> {
        read_one_byte(&mut self.inner)
    }
}
fn read_one_byte(reader: &mut Read) -> Option<Result<u8>> {
    let mut buf = [0];
    loop {
        return match reader.read(&mut buf) {
            Ok(0) => None,
            Ok(..) => Some(Ok(buf[0])),
            Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => Some(Err(e)),
        };
    }
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
}
impl Error {
    pub fn new<E>(kind: ErrorKind, _: E) -> Error {
        Error { kind }
    }
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    Interrupted,
    InvalidData,
    Other,
    UnexpectedEof,
}
