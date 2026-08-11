//! ELF64 parser and in-memory loader stub for user binaries.
//!
//! Full page mapping and ring-3 entry require M3 paging and M5 syscalls.
//! This module validates headers and reports load metadata for the M6 demo.

#![deny(missing_docs)]

/// ELF magic bytes `\x7FELF`.
pub const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];

/// ELF class: 64-bit.
const ELF_CLASS_64: u8 = 2;
/// ELF data encoding: little-endian.
const ELF_DATA_LSB: u8 = 1;
/// Executable file type.
const ET_EXEC: u16 = 2;
/// x86_64 machine type.
const EM_X86_64: u16 = 0x3E;

/// Parsed ELF load metadata (subset needed for M6).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElfLoadInfo {
    /// Program entry point virtual address.
    pub entry: u64,
    /// Minimum virtual address covered by loadable segments.
    pub load_vaddr_min: u64,
    /// Maximum virtual address covered by loadable segments.
    pub load_vaddr_max: u64,
    /// Total bytes required for loaded segments in memory.
    pub mem_size: u64,
    /// Number of PT_LOAD segments.
    pub load_segments: usize,
}

/// Errors while parsing or loading an ELF image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ElfError {
    /// Image too small to contain an ELF header.
    TooSmall,
    /// Magic, class, endianness, or machine type mismatch.
    BadHeader,
    /// Unsupported ELF type (not ET_EXEC).
    UnsupportedType,
    /// Program header table is out of bounds or malformed.
    BadPhdr,
    /// No PT_LOAD segments found.
    NoLoadSegments,
}

impl ElfError {
    /// Short diagnostic label for serial logging.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TooSmall => "too small",
            Self::BadHeader => "bad header",
            Self::UnsupportedType => "unsupported type",
            Self::BadPhdr => "bad phdr",
            Self::NoLoadSegments => "no load segments",
        }
    }
}

/// Parses an ELF64 image and returns load metadata without copying bytes.
pub fn parse_elf64(data: &[u8]) -> Result<ElfLoadInfo, ElfError> {
    if data.len() < 64 {
        return Err(ElfError::TooSmall);
    }

    if data[0..4] != ELF_MAGIC {
        return Err(ElfError::BadHeader);
    }
    if data[4] != ELF_CLASS_64 || data[5] != ELF_DATA_LSB {
        return Err(ElfError::BadHeader);
    }

    let e_type = u16::from_le_bytes([data[16], data[17]]);
    if e_type != ET_EXEC {
        return Err(ElfError::UnsupportedType);
    }

    let e_machine = u16::from_le_bytes([data[18], data[19]]);
    if e_machine != EM_X86_64 {
        return Err(ElfError::BadHeader);
    }

    let e_entry = u64::from_le_bytes(data[24..32].try_into().expect("e_entry"));
    let e_phoff = u64::from_le_bytes(data[32..40].try_into().expect("e_phoff"));
    let e_phentsize = u16::from_le_bytes([data[54], data[55]]) as u64;
    let e_phnum = u16::from_le_bytes([data[56], data[57]]) as usize;

    if e_phentsize < 56 || e_phnum == 0 {
        return Err(ElfError::BadPhdr);
    }

    let mut load_vaddr_min = u64::MAX;
    let mut load_vaddr_max = 0u64;
    let mut mem_size = 0u64;
    let mut load_segments = 0usize;

    for i in 0..e_phnum {
        let off = e_phoff.checked_add(i as u64 * e_phentsize).ok_or(ElfError::BadPhdr)?;
        let end = off.checked_add(56).ok_or(ElfError::BadPhdr)?;
        if end as usize > data.len() {
            return Err(ElfError::BadPhdr);
        }

        let phdr = &data[off as usize..end as usize];
        let p_type = u32::from_le_bytes(phdr[0..4].try_into().expect("p_type"));
        if p_type != 1 {
            continue; // PT_LOAD
        }

        let p_vaddr = u64::from_le_bytes(phdr[16..24].try_into().expect("p_vaddr"));
        let p_memsz = u64::from_le_bytes(phdr[40..48].try_into().expect("p_memsz"));

        load_segments += 1;
        load_vaddr_min = load_vaddr_min.min(p_vaddr);
        load_vaddr_max = load_vaddr_max.max(p_vaddr.saturating_add(p_memsz));
        mem_size = mem_size.saturating_add(p_memsz);
    }

    if load_segments == 0 {
        return Err(ElfError::NoLoadSegments);
    }

    Ok(ElfLoadInfo { entry: e_entry, load_vaddr_min, load_vaddr_max, mem_size, load_segments })
}

/// Validates ELF metadata and returns the entry point.
///
/// Does **not** map pages or transfer control — paging (M3) and ring-3 (M5) are
/// required before user binaries can run.
pub fn load_elf_stub(data: &[u8]) -> Result<u64, ElfError> {
    let info = parse_elf64(data)?;
    let _ = info;
    Ok(info.entry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_truncated_image() {
        assert_eq!(parse_elf64(&[0u8; 32]), Err(ElfError::TooSmall));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = [0u8; 128];
        buf[4] = ELF_CLASS_64;
        buf[5] = ELF_DATA_LSB;
        assert_eq!(parse_elf64(&buf), Err(ElfError::BadHeader));
    }
}
