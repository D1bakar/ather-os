//! Minimal FAT32 super-floppy image writer for UEFI ESP trees.

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

const SECTOR_SIZE: u32 = 512;
const SECTORS_PER_CLUSTER: u32 = 8;
const RESERVED_SECTORS: u32 = 32;
const NUM_FATS: u32 = 2;
const ROOT_CLUSTER: u32 = 2;

pub fn build_image(path: &Path, size_bytes: u64, files: &[(&str, Vec<u8>)]) -> Result<(), String> {
    if size_bytes % SECTOR_SIZE as u64 != 0 {
        return Err("image size must be a multiple of 512 bytes".into());
    }

    let total_sectors = (size_bytes / SECTOR_SIZE as u64) as u32;
    let data_sectors = total_sectors - RESERVED_SECTORS;
    let cluster_count = data_sectors / SECTORS_PER_CLUSTER;
    let fat_size = ((cluster_count + 2) * 4).div_ceil(SECTOR_SIZE);
    let first_data_sector = RESERVED_SECTORS + NUM_FATS * fat_size;
    let data_start = first_data_sector as u64 * SECTOR_SIZE as u64;

    let mut image = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    image.set_len(size_bytes).map_err(|e| e.to_string())?;

    write_boot_sector(
        &mut image,
        total_sectors,
        SECTORS_PER_CLUSTER,
        RESERVED_SECTORS,
        NUM_FATS,
        fat_size,
        ROOT_CLUSTER,
    )?;
    write_fsinfo(&mut image, cluster_count + 2)?;

    let fat_offset = RESERVED_SECTORS as u64 * SECTOR_SIZE as u64;
    init_fat(&mut image, fat_offset, fat_size, NUM_FATS, cluster_count + 2)?;

    let mut fs = FatWriter {
        image: &mut image,
        fat_offset,
        fat_size,
        num_fats: NUM_FATS,
        sectors_per_cluster: SECTORS_PER_CLUSTER,
        data_start,
        next_cluster: ROOT_CLUSTER + 1,
        cluster_count: cluster_count + 2,
    };

    let mut dirs: Vec<(String, u32)> = vec![("/".to_string(), ROOT_CLUSTER)];
    for (rel, data) in files {
        let parts: Vec<&str> = rel.split('/').collect();
        let file_name = *parts.last().expect("non-empty path");
        let parent = if parts.len() == 1 {
            ROOT_CLUSTER
        } else {
            let dir_path = parts[..parts.len() - 1].join("/");
            ensure_dir(&mut fs, &mut dirs, &dir_path)?
        };
        write_file(&mut fs, parent, file_name, data)?;
    }

    finalize_root(&mut fs, ROOT_CLUSTER)?;
    image.flush().map_err(|e| e.to_string())?;
    Ok(())
}

struct FatWriter<'a> {
    image: &'a mut File,
    fat_offset: u64,
    fat_size: u32,
    num_fats: u32,
    sectors_per_cluster: u32,
    data_start: u64,
    next_cluster: u32,
    cluster_count: u32,
}

fn write_boot_sector(
    image: &mut File,
    total_sectors: u32,
    sectors_per_cluster: u32,
    reserved_sectors: u32,
    num_fats: u32,
    fat_size: u32,
    root_cluster: u32,
) -> Result<(), String> {
    let mut sector = [0u8; 512];
    sector[0..3].copy_from_slice(b"\xEB\x58\x90");
    sector[3..11].copy_from_slice(b"MSWIN4.1");
    sector[11..13].copy_from_slice(&(SECTOR_SIZE as u16).to_le_bytes());
    sector[13] = sectors_per_cluster as u8;
    sector[14..16].copy_from_slice(&(reserved_sectors as u16).to_le_bytes());
    sector[16] = num_fats as u8;
    sector[17..19].copy_from_slice(&0u16.to_le_bytes());
    sector[19..21].copy_from_slice(&0u16.to_le_bytes());
    sector[21] = 0xF8;
    sector[22..24].copy_from_slice(&0u16.to_le_bytes());
    sector[24..26].copy_from_slice(&0u16.to_le_bytes());
    sector[26..28].copy_from_slice(&0u16.to_le_bytes());
    sector[28..32].copy_from_slice(&0u32.to_le_bytes());
    sector[32..36].copy_from_slice(&total_sectors.to_le_bytes());
    sector[36..40].copy_from_slice(&fat_size.to_le_bytes());
    sector[40..42].copy_from_slice(&0u16.to_le_bytes());
    sector[42..44].copy_from_slice(&0u16.to_le_bytes());
    sector[44..48].copy_from_slice(&root_cluster.to_le_bytes());
    sector[48..52].copy_from_slice(&2u32.to_le_bytes());
    sector[52..54].copy_from_slice(&1u16.to_le_bytes());
    sector[64..66].copy_from_slice(&0x0029u16.to_le_bytes());
    sector[66..70].copy_from_slice(&0x1234_5678u32.to_le_bytes());
    sector[71..82].copy_from_slice(b"AETHEROS   ");
    sector[82..90].copy_from_slice(b"FAT32   ");
    sector[510..512].copy_from_slice(&0xAA55u16.to_le_bytes());

    image.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    image.write_all(&sector).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_fsinfo(image: &mut File, total_clusters: u32) -> Result<(), String> {
    let mut sector = [0u8; 512];
    sector[0..4].copy_from_slice(b"RRaA");
    sector[484..488].copy_from_slice(b"rrAa");
    sector[488..492].copy_from_slice(&0xFFFF_FFFCu32.to_le_bytes());
    sector[492..496].copy_from_slice(&3u32.to_le_bytes());
    let _ = total_clusters;
    sector[510..512].copy_from_slice(&0xAA55u16.to_le_bytes());
    image.seek(SeekFrom::Start(512)).map_err(|e| e.to_string())?;
    image.write_all(&sector).map_err(|e| e.to_string())?;
    Ok(())
}

fn init_fat(
    image: &mut File,
    fat_offset: u64,
    fat_size: u32,
    num_fats: u32,
    total_clusters: u32,
) -> Result<(), String> {
    let fat_bytes = fat_size as u64 * SECTOR_SIZE as u64;
    let mut fat = vec![0u8; fat_bytes as usize];
    put_fat32(&mut fat, 0, 0x0FFF_FFF8);
    put_fat32(&mut fat, 1, 0x0FFF_FFFF);
    put_fat32(&mut fat, ROOT_CLUSTER, 0x0FFF_FFFF);
    for fat_index in 0..num_fats {
        let offset = fat_offset + fat_index as u64 * fat_bytes;
        image.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
        image.write_all(&fat).map_err(|e| e.to_string())?;
    }
    let _ = total_clusters;
    Ok(())
}

fn put_fat32(fat: &mut [u8], cluster: u32, value: u32) {
    let offset = cluster as usize * 4;
    fat[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn ensure_dir(
    fs: &mut FatWriter<'_>,
    dirs: &mut Vec<(String, u32)>,
    path: &str,
) -> Result<u32, String> {
    if let Some((_, cluster)) = dirs.iter().find(|(p, _)| p == path) {
        return Ok(*cluster);
    }

    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    let mut current_path = String::new();
    let mut parent = ROOT_CLUSTER;
    for part in parts {
        if !current_path.is_empty() {
            current_path.push('/');
        }
        current_path.push_str(part);
        if let Some((_, cluster)) = dirs.iter().find(|(p, _)| p == &current_path) {
            parent = *cluster;
            continue;
        }
        parent = create_dir_entry(fs, parent, part, true)?;
        dirs.push((current_path.clone(), parent));
    }
    Ok(parent)
}

fn create_dir_entry(
    fs: &mut FatWriter<'_>,
    parent_cluster: u32,
    name: &str,
    is_dir: bool,
) -> Result<u32, String> {
    let cluster = fs.alloc_cluster()?;
    if is_dir {
        init_directory_cluster(fs, cluster, parent_cluster)?;
    }
    append_dir_entries(fs, parent_cluster, name, cluster, is_dir, 0)?;
    Ok(cluster)
}

fn write_file(
    fs: &mut FatWriter<'_>,
    parent_cluster: u32,
    name: &str,
    data: &[u8],
) -> Result<(), String> {
    let cluster = fs.alloc_cluster()?;
    write_cluster_chain(fs, cluster, data)?;
    append_dir_entries(fs, parent_cluster, name, cluster, false, data.len() as u32)?;
    Ok(())
}

fn init_directory_cluster(
    fs: &mut FatWriter<'_>,
    cluster: u32,
    parent_cluster: u32,
) -> Result<(), String> {
    let mut buf = vec![0u8; (fs.sectors_per_cluster * SECTOR_SIZE) as usize];
    write_dot_entries(&mut buf, cluster, parent_cluster);
    write_cluster(fs, cluster, &buf)
}

fn write_dot_entries(buf: &mut [u8], cluster: u32, parent_cluster: u32) {
    let dot = *b".          ";
    let dotdot = *b"..         ";
    write_short_entry(buf, 0, &dot, cluster, true, 0);
    write_short_entry(buf, 32, &dotdot, parent_cluster, true, 0);
}

fn append_dir_entries(
    fs: &mut FatWriter<'_>,
    dir_cluster: u32,
    name: &str,
    first_cluster: u32,
    is_dir: bool,
    size: u32,
) -> Result<(), String> {
    let mut cluster = dir_cluster;
    loop {
        let mut buf = read_cluster(fs, cluster)?;
        if let Some(offset) = find_free_dir_slot(&buf, name)? {
            write_lfn_entries(&mut buf, offset, name)?;
            let short = to_short_name(name);
            write_short_entry(
                &mut buf,
                offset + lfn_slots(name) * 32,
                &short,
                first_cluster,
                is_dir,
                size,
            );
            write_cluster(fs, cluster, &buf)?;
            return Ok(());
        }
        let next = fs.fat_entry(cluster)?;
        if next >= fs.cluster_count {
            let new_cluster = fs.alloc_cluster()?;
            fs.set_fat_entry(cluster, new_cluster)?;
            fs.set_fat_entry(new_cluster, 0x0FFF_FFFF)?;
            init_directory_cluster(fs, new_cluster, dir_cluster)?;
            cluster = new_cluster;
        } else {
            cluster = next;
        }
    }
}

fn finalize_root(fs: &mut FatWriter<'_>, root_cluster: u32) -> Result<(), String> {
    let mut buf = read_cluster(fs, root_cluster)?;
    if buf[0] == 0 {
        write_dot_entries(&mut buf, root_cluster, root_cluster);
        write_cluster(fs, root_cluster, &buf)?;
    }
    Ok(())
}

fn find_free_dir_slot(buf: &[u8], name: &str) -> Result<Option<usize>, String> {
    let needed = (lfn_slots(name) + 1) * 32;
    let mut offset = 0usize;
    while offset + needed <= buf.len() {
        if (buf[offset] == 0 || buf[offset] == 0xE5) && slot_run_free(buf, offset, needed) {
            return Ok(Some(offset));
        }
        offset += 32;
    }
    Ok(None)
}

fn slot_run_free(buf: &[u8], start: usize, bytes: usize) -> bool {
    for offset in (start..start + bytes).step_by(32) {
        let b = buf[offset];
        if b != 0 && b != 0xE5 {
            return false;
        }
    }
    true
}

fn lfn_slots(name: &str) -> usize {
    (name.chars().count() + 12) / 13
}

fn write_lfn_entries(buf: &mut [u8], start: usize, name: &str) -> Result<(), String> {
    let chars: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let slots = lfn_slots(name);
    let short = to_short_name(name);
    let checksum = lfn_checksum(&short);
    for slot in 0..slots {
        let offset = start + slot * 32;
        let seq = if slot + 1 == slots { (slot + 1) as u8 | 0x40 } else { (slot + 1) as u8 };
        buf[offset] = seq;
        buf[offset + 11] = 0x0F;
        buf[offset + 12] = 0;
        buf[offset + 13] = checksum;
        buf[offset + 26..offset + 28].copy_from_slice(&0u16.to_le_bytes());
        buf[offset + 28..offset + 32].copy_from_slice(&0u32.to_le_bytes());

        let base = slot * 13;
        write_lfn_chunk(&mut buf[offset + 1..offset + 11], &chars, base, 5);
        write_lfn_chunk(&mut buf[offset + 14..offset + 26], &chars, base + 5, 6);
        write_lfn_chunk(&mut buf[offset + 28..offset + 32], &chars, base + 11, 2);
    }
    Ok(())
}

fn write_lfn_chunk(out: &mut [u8], chars: &[u16], start: usize, count: usize) {
    for i in 0..count {
        let value = chars.get(start + i).copied().unwrap_or(0xFFFF);
        out[i * 2..i * 2 + 2].copy_from_slice(&value.to_le_bytes());
    }
}

fn lfn_checksum(short: &[u8; 11]) -> u8 {
    short.iter().fold(0u8, |sum, &b| sum.rotate_left(1).wrapping_add(b))
}

fn to_short_name(name: &str) -> [u8; 11] {
    let base = name.rsplit('/').next().unwrap_or(name);
    let mut short = [0x20u8; 11];
    let (stem, ext) = split_stem_ext(base);
    if stem.len() <= 8 && ext.len() <= 3 {
        copy_short_part(&mut short[0..8], stem);
        copy_short_part(&mut short[8..11], ext);
        return short;
    }
    let stem_part = &stem[..stem.len().min(6)];
    copy_short_part(&mut short[0..8], &format!("{stem_part}~1"));
    copy_short_part(&mut short[8..11], ext);
    short
}

fn split_stem_ext(name: &str) -> (&str, &str) {
    match name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && ext.len() <= 3 => (stem, ext),
        _ => (name, ""),
    }
}

fn copy_short_part(out: &mut [u8], part: &str) {
    for (i, ch) in part.chars().take(out.len()).map(|c| c as u8).enumerate() {
        out[i] = ch;
    }
}

fn write_short_entry(
    buf: &mut [u8],
    offset: usize,
    short_name: &[u8; 11],
    cluster: u32,
    is_dir: bool,
    size: u32,
) {
    let entry = &mut buf[offset..offset + 32];
    entry[..11].copy_from_slice(short_name);
    entry[11] = if is_dir { 0x10 } else { 0x20 };
    entry[26..28].copy_from_slice(&(cluster as u16).to_le_bytes());
    entry[20..22].copy_from_slice(&((cluster >> 16) as u16).to_le_bytes());
    entry[28..32].copy_from_slice(&size.to_le_bytes());
}

impl FatWriter<'_> {
    fn alloc_cluster(&mut self) -> Result<u32, String> {
        if self.next_cluster >= self.cluster_count {
            return Err("FAT32 image full".into());
        }
        let cluster = self.next_cluster;
        self.next_cluster += 1;
        self.set_fat_entry(cluster, 0x0FFF_FFFF)?;
        Ok(cluster)
    }

    fn fat_entry(&mut self, cluster: u32) -> Result<u32, String> {
        let offset = self.fat_offset + cluster as u64 * 4;
        let mut buf = [0u8; 4];
        self.image.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
        self.image.read_exact_slice(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn set_fat_entry(&mut self, cluster: u32, value: u32) -> Result<(), String> {
        for fat_index in 0..self.num_fats {
            let offset = self.fat_offset
                + fat_index as u64 * self.fat_size as u64 * SECTOR_SIZE as u64
                + cluster as u64 * 4;
            self.image.seek(SeekFrom::Start(offset)).map_err(|e| e.to_string())?;
            self.image.write_all(&value.to_le_bytes()).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

trait ReadExactSlice {
    fn read_exact_slice(&mut self, buf: &mut [u8]) -> Result<(), String>;
}

impl ReadExactSlice for File {
    fn read_exact_slice(&mut self, buf: &mut [u8]) -> Result<(), String> {
        use std::io::Read;
        self.read_exact(buf).map_err(|e| e.to_string())
    }
}

fn cluster_offset(fs: &FatWriter<'_>, cluster: u32) -> u64 {
    fs.data_start + (cluster - 2) as u64 * fs.sectors_per_cluster as u64 * SECTOR_SIZE as u64
}

fn read_cluster(fs: &mut FatWriter<'_>, cluster: u32) -> Result<Vec<u8>, String> {
    let size = (fs.sectors_per_cluster * SECTOR_SIZE) as usize;
    let mut buf = vec![0u8; size];
    fs.image.seek(SeekFrom::Start(cluster_offset(fs, cluster))).map_err(|e| e.to_string())?;
    fs.image.read_exact_slice(&mut buf)?;
    Ok(buf)
}

fn write_cluster(fs: &mut FatWriter<'_>, cluster: u32, data: &[u8]) -> Result<(), String> {
    fs.image.seek(SeekFrom::Start(cluster_offset(fs, cluster))).map_err(|e| e.to_string())?;
    fs.image.write_all(data).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_cluster_chain(fs: &mut FatWriter<'_>, first: u32, data: &[u8]) -> Result<(), String> {
    let cluster_bytes = (fs.sectors_per_cluster * SECTOR_SIZE) as usize;
    let mut cluster = first;
    let mut written = 0usize;
    loop {
        let end = (written + cluster_bytes).min(data.len());
        let chunk = &data[written..end];
        let mut buf = vec![0u8; cluster_bytes];
        buf[..chunk.len()].copy_from_slice(chunk);
        write_cluster(fs, cluster, &buf)?;
        written = end;
        if written >= data.len() {
            fs.set_fat_entry(cluster, 0x0FFF_FFFF)?;
            break;
        }
        let next = fs.alloc_cluster()?;
        fs.set_fat_entry(cluster, next)?;
        cluster = next;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn builds_small_fat32_image() {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("aether-test-{stamp}.img"));
        let files = [("EFI/BOOT/BOOTX64.EFI".to_string(), vec![0x4D, 0x5A, 0x90, 0x00])];
        let refs: Vec<(&str, Vec<u8>)> =
            files.iter().map(|(n, d)| (n.as_str(), d.clone())).collect();
        build_image(&path, 16 * 1024 * 1024, &refs).expect("build image");
        assert!(path.metadata().unwrap().len() > 0);
        let _ = std::fs::remove_file(path);
    }
}
