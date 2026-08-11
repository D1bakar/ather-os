//! Host integration tests for `libs/` crates used by the kernel.

use aether_kernel::{AetherVec, IoError, Read, SpinMutex, StrWriter, Write, WriteStr};

#[test]
fn kernel_reexports_spin_mutex() {
    let lock = SpinMutex::new(10u32);
    *lock.lock() += 5;
    assert_eq!(*lock.lock(), 15);
}

#[test]
fn kernel_reexports_aether_vec() {
    let mut v = AetherVec::new();
    v.push(1);
    v.push(2);
    assert_eq!(v.len(), 2);
    assert_eq!(v.as_slice(), &[1, 2]);
}

struct EchoReader {
    data: &'static [u8],
    pos: usize,
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

struct CollectWriter {
    buf: AetherVec<u8>,
}

impl Write for CollectWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        for &b in buf {
            self.buf.push(b);
        }
        Ok(buf.len())
    }
}

#[test]
fn io_traits_read_write_roundtrip() {
    let mut reader = EchoReader { data: b"abc", pos: 0 };
    let mut out = [0u8; 2];
    assert_eq!(reader.read(&mut out).unwrap(), 2);
    assert_eq!(&out, b"ab");

    let mut writer = CollectWriter { buf: AetherVec::new() };
    writer.write_all(b"xy").unwrap();
    assert_eq!(writer.buf.as_slice(), b"xy");
}

struct StrSink {
    text: AetherVec<u8>,
}

impl WriteStr for StrSink {
    fn write_str(&mut self, s: &str) -> Result<(), IoError> {
        self.text.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

#[test]
fn write_str_trait_via_adapter() {
    let mut sink = StrWriter::new(StrSink { text: AetherVec::new() });
    sink.write_all(b"ok").unwrap();
    assert_eq!(sink.inner().text.as_slice(), b"ok");
}
