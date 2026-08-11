//! Core read/write traits.

use crate::IoError;

/// A byte-oriented input source.
pub trait Read {
    /// Reads up to `buf.len()` bytes into `buf`.
    ///
    /// Returns the number of bytes read, or [`IoError::EndOfStream`] when no
    /// data is available and the stream has ended.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError>;
}

/// A byte-oriented output sink.
pub trait Write {
    /// Writes `buf` to the sink, returning the number of bytes accepted.
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError>;

    /// Flushes buffered output, if any.
    fn flush(&mut self) -> Result<(), IoError> {
        Ok(())
    }

    /// Writes the entire buffer, retrying until all bytes are accepted.
    fn write_all(&mut self, mut buf: &[u8]) -> Result<(), IoError> {
        while !buf.is_empty() {
            let written = self.write(buf)?;
            if written == 0 {
                return Err(IoError::WouldBlock);
            }
            buf = &buf[written..];
        }
        Ok(())
    }
}

/// Adapter that implements [`Write`] for anything exposing `write_str`.
pub struct StrWriter<W> {
    inner: W,
}

#[allow(dead_code)] // public adapter API; used by downstream crates once wired
impl<W> StrWriter<W> {
    /// Wraps `inner` as a [`Write`] adapter.
    pub const fn new(inner: W) -> Self {
        Self { inner }
    }

    /// Returns a shared reference to the inner writer.
    pub const fn inner(&self) -> &W {
        &self.inner
    }

    /// Returns a mutable reference to the inner writer.
    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    /// Consumes the adapter and returns the inner writer.
    pub fn into_inner(self) -> W {
        self.inner
    }
}

/// Extension for writers that can emit UTF-8 text directly.
pub trait WriteStr {
    /// Writes a UTF-8 string.
    fn write_str(&mut self, s: &str) -> Result<(), IoError>;
}

impl<W: WriteStr> Write for StrWriter<W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        self.inner.write_str(core::str::from_utf8(buf).map_err(|_| IoError::InvalidInput)?)?;
        Ok(buf.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct SliceWriter {
        buf: alloc::vec::Vec<u8>,
    }

    impl Write for SliceWriter {
        fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
            self.buf.extend_from_slice(buf);
            Ok(buf.len())
        }
    }

    #[test]
    fn write_all_writes_every_byte() {
        let mut writer = SliceWriter { buf: alloc::vec::Vec::new() };
        writer.write_all(b"abc").unwrap();
        assert_eq!(writer.buf, b"abc");
    }

    struct EchoReader {
        pos: usize,
        data: &'static [u8],
    }

    impl Read for EchoReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
            if self.pos >= self.data.len() {
                return Err(IoError::EndOfStream);
            }
            let n = core::cmp::min(buf.len(), self.data.len() - self.pos);
            buf[..n].copy_from_slice(&self.data[self.pos..self.pos + n]);
            self.pos += n;
            Ok(n)
        }
    }

    #[test]
    fn str_writer_adapts_write_str() {
        struct Sink(alloc::vec::Vec<u8>);
        impl WriteStr for Sink {
            fn write_str(&mut self, s: &str) -> Result<(), IoError> {
                self.0.extend_from_slice(s.as_bytes());
                Ok(())
            }
        }

        let mut writer = StrWriter::new(Sink(alloc::vec::Vec::new()));
        writer.write_all(b"hi").unwrap();
        assert_eq!(writer.inner().0, b"hi");
        writer.inner_mut().0.push(b'!');
        assert_eq!(writer.into_inner().0, b"hi!");
    }

    #[test]
    fn read_fills_buffer() {
        let mut reader = EchoReader { pos: 0, data: b"hello" };
        let mut out = [0u8; 3];
        assert_eq!(reader.read(&mut out).unwrap(), 3);
        assert_eq!(&out, b"hel");
    }
}
