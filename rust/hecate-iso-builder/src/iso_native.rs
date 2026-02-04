//! Native Rust ISO 9660 creation module
//! 
//! This module provides pure Rust ISO 9660 filesystem creation
//! without any external dependencies.

use std::fs::File;
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use anyhow::Result;
use walkdir::WalkDir;
use chrono::{Utc, Datelike, Timelike};

const SECTOR_SIZE: usize = 2048;
const SYSTEM_AREA_SIZE: usize = 16 * SECTOR_SIZE; // 32KB reserved for boot

/// ISO 9660 Volume Descriptor types
#[repr(u8)]
enum VolumeDescriptorType {
    BootRecord = 0,
    PrimaryVolume = 1,
    SupplementaryVolume = 2,
    VolumePartition = 3,
    SetTerminator = 255,
}

/// Native ISO 9660 builder
pub struct NativeIsoBuilder {
    volume_id: String,
    publisher: String,
    preparer: String,
    files: Vec<IsoFileEntry>,
    directories: Vec<IsoDirEntry>,
}

struct IsoFileEntry {
    path: PathBuf,
    iso_path: String,
    size: u64,
    start_sector: u32,
}

struct IsoDirEntry {
    path: String,
    parent: Option<String>,
    start_sector: u32,
}

impl NativeIsoBuilder {
    pub fn new(volume_id: String) -> Self {
        Self {
            volume_id: volume_id.to_uppercase(),
            publisher: "HECATEOS".to_string(),
            preparer: "HECATE-ISO-BUILDER".to_string(),
            files: Vec::new(),
            directories: Vec::new(),
        }
    }
    
    /// Add a directory tree to the ISO
    pub fn add_directory_tree(&mut self, source: &Path, iso_path: &str) -> Result<()> {
        for entry in WalkDir::new(source) {
            let entry = entry?;
            let path = entry.path();
            let relative = path.strip_prefix(source)?;
            let iso_name = format!("{}/{}", iso_path, relative.display()).replace("//", "/");
            
            if path.is_dir() {
                self.directories.push(IsoDirEntry {
                    path: iso_name.to_uppercase(),
                    parent: None,
                    start_sector: 0,
                });
            } else if path.is_file() {
                let metadata = path.metadata()?;
                self.files.push(IsoFileEntry {
                    path: path.to_path_buf(),
                    iso_path: iso_name.to_uppercase(),
                    size: metadata.len(),
                    start_sector: 0,
                });
            }
        }
        Ok(())
    }
    
    /// Create the ISO file
    pub fn build(&mut self, output: &Path) -> Result<()> {
        let mut iso = File::create(output)?;
        
        // Write system area (boot area)
        self.write_system_area(&mut iso)?;
        
        // Write primary volume descriptor
        self.write_primary_volume_descriptor(&mut iso)?;
        
        // Write volume descriptor set terminator
        self.write_volume_set_terminator(&mut iso)?;
        
        // Calculate file positions
        let mut current_sector = 19; // After volume descriptors
        
        // Write directory records
        for dir in &mut self.directories {
            dir.start_sector = current_sector;
            current_sector += 1;
        }
        
        // Write files data
        for file in &mut self.files {
            file.start_sector = current_sector;
            let sectors_needed = ((file.size + SECTOR_SIZE as u64 - 1) / SECTOR_SIZE as u64) as u32;
            current_sector += sectors_needed;
        }
        
        // Write actual directory structures
        self.write_directory_records(&mut iso)?;
        
        // Write file data
        self.write_file_data(&mut iso)?;
        
        Ok(())
    }
    
    fn write_system_area(&self, iso: &mut File) -> Result<()> {
        // Write 32KB of zeros for system/boot area
        let zeros = vec![0u8; SYSTEM_AREA_SIZE];
        iso.write_all(&zeros)?;
        Ok(())
    }
    
    fn write_primary_volume_descriptor(&self, iso: &mut File) -> Result<()> {
        let mut descriptor = vec![0u8; SECTOR_SIZE];
        
        // Volume descriptor type
        descriptor[0] = VolumeDescriptorType::PrimaryVolume as u8;
        
        // Standard identifier "CD001"
        descriptor[1..6].copy_from_slice(b"CD001");
        
        // Version
        descriptor[6] = 1;
        
        // System identifier (32 bytes)
        let system_id = b"HECATEOS                        ";
        descriptor[8..40].copy_from_slice(&system_id[..32]);
        
        // Volume identifier (32 bytes)
        let vol_id = format!("{:32}", self.volume_id);
        descriptor[40..72].copy_from_slice(vol_id.as_bytes());
        
        // Volume space size (number of logical blocks)
        let total_sectors = self.calculate_total_sectors();
        self.write_both_endian_32(&mut descriptor[80..88], total_sectors);
        
        // Volume set size
        self.write_both_endian_16(&mut descriptor[120..124], 1);
        
        // Volume sequence number
        self.write_both_endian_16(&mut descriptor[124..128], 1);
        
        // Logical block size
        self.write_both_endian_16(&mut descriptor[128..132], SECTOR_SIZE as u16);
        
        // Path table size (simplified)
        let path_table_size = self.calculate_path_table_size();
        self.write_both_endian_32(&mut descriptor[132..140], path_table_size);
        
        // Location of L path table
        descriptor[140..144].copy_from_slice(&20u32.to_le_bytes());
        
        // Location of M path table  
        descriptor[148..152].copy_from_slice(&22u32.to_be_bytes());
        
        // Root directory record (34 bytes at offset 156)
        self.write_root_directory_record(&mut descriptor[156..190])?;
        
        // Volume set identifier (128 bytes)
        let volset = format!("{:128}", "HECATEOS_SET");
        descriptor[190..318].copy_from_slice(volset.as_bytes());
        
        // Publisher identifier (128 bytes)
        let publisher = format!("{:128}", self.publisher);
        descriptor[318..446].copy_from_slice(publisher.as_bytes());
        
        // Data preparer identifier (128 bytes)
        let preparer = format!("{:128}", self.preparer);
        descriptor[446..574].copy_from_slice(preparer.as_bytes());
        
        // Timestamps (17 bytes each: YYYYMMDDHHMMSSmm + null terminator)
        let now = Utc::now();
        let timestamp = self.format_timestamp(&now);
        let mut timestamp_bytes = [0u8; 17];
        let ts_bytes = timestamp.as_bytes();
        let len = ts_bytes.len().min(16);
        timestamp_bytes[..len].copy_from_slice(&ts_bytes[..len]);
        
        // Creation date
        descriptor[813..830].copy_from_slice(&timestamp_bytes);
        
        // Modification date  
        descriptor[830..847].copy_from_slice(&timestamp_bytes);
        
        // Effective date
        descriptor[864..881].copy_from_slice(&timestamp_bytes);
        
        // File structure version
        descriptor[881] = 1;
        
        iso.write_all(&descriptor)?;
        Ok(())
    }
    
    fn write_volume_set_terminator(&self, iso: &mut File) -> Result<()> {
        let mut terminator = vec![0u8; SECTOR_SIZE];
        terminator[0] = VolumeDescriptorType::SetTerminator as u8;
        terminator[1..6].copy_from_slice(b"CD001");
        terminator[6] = 1;
        
        iso.write_all(&terminator)?;
        Ok(())
    }
    
    fn write_directory_records(&self, iso: &mut File) -> Result<()> {
        // Simplified: write basic directory structure
        // In a full implementation, this would write proper directory records
        for _ in &self.directories {
            let dir_record = vec![0u8; SECTOR_SIZE];
            iso.write_all(&dir_record)?;
        }
        Ok(())
    }
    
    fn write_file_data(&self, iso: &mut File) -> Result<()> {
        for file in &self.files {
            // Seek to the file's start sector
            iso.seek(SeekFrom::Start((file.start_sector as u64) * SECTOR_SIZE as u64))?;
            
            // Copy file data
            let mut source = File::open(&file.path)?;
            let mut buffer = vec![0u8; SECTOR_SIZE];
            
            loop {
                let bytes_read = source.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                
                // Pad last sector with zeros
                if bytes_read < SECTOR_SIZE {
                    buffer[bytes_read..].fill(0);
                }
                
                iso.write_all(&buffer)?;
                
                if bytes_read < SECTOR_SIZE {
                    break;
                }
            }
        }
        Ok(())
    }
    
    fn write_root_directory_record(&self, buffer: &mut [u8]) -> Result<()> {
        // Length of directory record
        buffer[0] = 34;
        
        // Extended attribute record length
        buffer[1] = 0;
        
        // Location of extent (root at sector 19)
        self.write_both_endian_32(&mut buffer[2..10], 19);
        
        // Data length (one sector for root)
        self.write_both_endian_32(&mut buffer[10..18], SECTOR_SIZE as u32);
        
        // Recording date and time
        let now = Utc::now();
        buffer[18] = (now.year() - 1900) as u8;
        buffer[19] = now.month() as u8;
        buffer[20] = now.day() as u8;
        buffer[21] = now.hour() as u8;
        buffer[22] = now.minute() as u8;
        buffer[23] = now.second() as u8;
        buffer[24] = 0; // GMT offset
        
        // File flags (directory)
        buffer[25] = 0x02;
        
        // File unit size
        buffer[26] = 0;
        
        // Interleave gap size
        buffer[27] = 0;
        
        // Volume sequence number
        self.write_both_endian_16(&mut buffer[28..32], 1);
        
        // Length of file identifier
        buffer[32] = 1;
        
        // File identifier (0x00 for root)
        buffer[33] = 0;
        
        Ok(())
    }
    
    fn write_both_endian_16(&self, buffer: &mut [u8], value: u16) {
        buffer[0..2].copy_from_slice(&value.to_le_bytes());
        buffer[2..4].copy_from_slice(&value.to_be_bytes());
    }
    
    fn write_both_endian_32(&self, buffer: &mut [u8], value: u32) {
        buffer[0..4].copy_from_slice(&value.to_le_bytes());
        buffer[4..8].copy_from_slice(&value.to_be_bytes());
    }
    
    fn calculate_total_sectors(&self) -> u32 {
        let mut sectors = 19; // System area + volume descriptors
        sectors += self.directories.len() as u32;
        
        for file in &self.files {
            sectors += ((file.size + SECTOR_SIZE as u64 - 1) / SECTOR_SIZE as u64) as u32;
        }
        
        sectors
    }
    
    fn calculate_path_table_size(&self) -> u32 {
        // Simplified calculation
        (self.directories.len() * 12) as u32
    }
    
    fn format_timestamp(&self, dt: &chrono::DateTime<Utc>) -> String {
        format!(
            "{:04}{:02}{:02}{:02}{:02}{:02}00",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        )
    }
}

/// Simplified ISO extraction (using existing tools for now)
pub fn extract_iso(iso_path: &Path, output_dir: &Path) -> Result<()> {
    // For extraction, we still use 7z if available since reading ISO is complex
    std::fs::create_dir_all(output_dir)?;
    
    let output = std::process::Command::new("7z")
        .arg("x")
        .arg(format!("-o{}", output_dir.display()))
        .arg("-y")
        .arg(iso_path)
        .output();
    
    match output {
        Ok(result) if result.status.success() => Ok(()),
        _ => {
            // Fallback message
            anyhow::bail!(
                "ISO extraction requires 7z for now. Install with: sudo apt-get install p7zip-full\n\
                Future versions will include native extraction."
            )
        }
    }
}