//! Kernel image signature verification stub.
//!
//! Full Ed25519 (or Secure Boot) verification is planned; this module defines
//! the hook point and accepts unsigned development kernels.

/// Trailing footer magic written by future signing tooling (`b"AETHRSIG\0"`).
const SIGNATURE_FOOTER_MAGIC: &[u8; 8] = b"AETHRSIG";

/// Outcome of locating a signature marker in the kernel image.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SignatureMarker {
    /// No marker present — unsigned development image.
    Absent,
    /// Footer or section marker found; crypto verify not yet implemented.
    PresentStub,
}

/// Validates the kernel image signature hook (stub).
///
/// Returns `Ok(())` for unsigned kernels while signing infrastructure is absent.
/// Returns `Err(())` only when the image is not a valid ELF64 object.
pub fn verify_kernel_signature(data: &[u8]) -> Result<(), ()> {
    if data.len() < 64 || data[0..4] != [0x7F, b'E', b'L', b'F'] {
        log::error!("signature stub: file is not a valid ELF header");
        return Err(());
    }
    if data[4] != 2 || data[5] != 1 {
        log::error!("signature stub: expected ELF64 little-endian kernel");
        return Err(());
    }

    match locate_signature_marker(data) {
        SignatureMarker::PresentStub => {
            log::info!(
                "signature stub: marker found; cryptographic verify not enabled (accepting)"
            );
        }
        SignatureMarker::Absent => {
            log::warn!(
                "signature stub: kernel is unsigned; boot allowed while signing is disabled"
            );
        }
    }

    Ok(())
}

fn locate_signature_marker(data: &[u8]) -> SignatureMarker {
    if data.len() >= SIGNATURE_FOOTER_MAGIC.len()
        && data[data.len() - SIGNATURE_FOOTER_MAGIC.len()..] == *SIGNATURE_FOOTER_MAGIC
    {
        return SignatureMarker::PresentStub;
    }

    // Scan section header names for a future `.aether_sig` section (stub).
    if let Some(name) = find_section_name(data, b".aether_sig") {
        if !name.is_empty() {
            return SignatureMarker::PresentStub;
        }
    }

    SignatureMarker::Absent
}

fn find_section_name<'a>(data: &'a [u8], target: &[u8]) -> Option<&'a [u8]> {
    if data.len() < 64 {
        return None;
    }
    let e_shoff = read_u64(data, 0x28);
    let e_shentsize = read_u16(data, 0x3A);
    let e_shnum = read_u16(data, 0x3C);
    let e_shstrndx = read_u16(data, 0x3E);

    if e_shentsize < 64 || e_shnum == 0 {
        return None;
    }

    let shstr_off = e_shoff + u64::from(e_shstrndx) * u64::from(e_shentsize);
    if usize::try_from(shstr_off + 64).ok().filter(|&n| n <= data.len()).is_none() {
        return None;
    }

    let shstr_offset = read_u64(data, shstr_off + 0x18) as usize;
    let shstr_size = read_u64(data, shstr_off + 0x20) as usize;
    if shstr_offset.saturating_add(shstr_size) > data.len() {
        return None;
    }

    for i in 0..e_shnum {
        let sh_off = e_shoff + u64::from(i) * u64::from(e_shentsize);
        if usize::try_from(sh_off + 64).ok().filter(|&n| n <= data.len()).is_none() {
            continue;
        }
        let name_idx = read_u32(data, sh_off) as usize;
        if name_idx >= shstr_size {
            continue;
        }
        let name_start = shstr_offset + name_idx;
        let name_end = data[shstr_offset..]
            .get(name_idx..)
            .and_then(|s| s.iter().position(|&b| b == 0))
            .map(|pos| shstr_offset + name_idx + pos)
            .unwrap_or(shstr_offset + shstr_size);
        if name_end > data.len() {
            continue;
        }
        let name = &data[name_start..name_end];
        if name == target {
            return Some(name);
        }
    }

    None
}

fn read_u16(data: &[u8], offset: u64) -> u16 {
    let i = offset as usize;
    u16::from_le_bytes([data[i], data[i + 1]])
}

fn read_u32(data: &[u8], offset: u64) -> u32 {
    let i = offset as usize;
    u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]])
}

fn read_u64(data: &[u8], offset: u64) -> u64 {
    let i = offset as usize;
    u64::from_le_bytes([
        data[i],
        data[i + 1],
        data[i + 2],
        data[i + 3],
        data[i + 4],
        data[i + 5],
        data[i + 6],
        data[i + 7],
    ])
}
