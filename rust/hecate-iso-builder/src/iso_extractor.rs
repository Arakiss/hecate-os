//! Native ISO 9660 extractor in pure Rust
//! This module can extract ISO files without external dependencies

use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom, BufReader};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt};

const SECTOR_SIZE: usize = 2048;
const VOLUME_DESCRIPTOR_SECTOR: u64 = 16;

/// ISO 9660 Volume Descriptor
#[derive(Debug)]
struct VolumeDescriptor {
    type_code: u8,
    identifier: [u8; 5],
    version: u8,
    volume_space_size: u32,
    root_directory_record: DirectoryRecord,
}

/// ISO 9660 Directory Record
#[derive(Debug, Clone)]
struct DirectoryRecord {
    length: u8,
    location: u32,
    data_length: u32,
    is_directory: bool,
    file_identifier: String,
}

/// Native ISO extractor
pub struct IsoExtractor {
    file: BufReader<File>,
    volume_descriptor: Option<VolumeDescriptor>,
}

impl IsoExtractor {
    /// Open an ISO file for extraction
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref())
            .context("Failed to open ISO file")?;
        
        let mut extractor = IsoExtractor {
            file: BufReader::new(file),
            volume_descriptor: None,
        };
        
        extractor.read_volume_descriptor()?;
        Ok(extractor)
    }
    
    /// Extract all files to the specified directory
    pub fn extract_all<P: AsRef<Path>>(&mut self, output_dir: P) -> Result<()> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir)?;
        
        let root = self.volume_descriptor
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No volume descriptor"))?
            .root_directory_record
            .clone();
        
        self.extract_directory(&root, output_dir, "")?;
        Ok(())
    }
    
    /// Read the primary volume descriptor
    fn read_volume_descriptor(&mut self) -> Result<()> {
        // Seek to volume descriptor location (sector 16)
        self.file.seek(SeekFrom::Start(VOLUME_DESCRIPTOR_SECTOR * SECTOR_SIZE as u64))?;
        
        let mut buffer = vec![0u8; SECTOR_SIZE];
        self.file.read_exact(&mut buffer)?;
        
        // Check volume descriptor type (1 = Primary)
        if buffer[0] != 1 {
            return Err(anyhow::anyhow!("Not a primary volume descriptor"));
        }
        
        // Check identifier "CD001"
        if &buffer[1..6] != b"CD001" {
            return Err(anyhow::anyhow!("Invalid ISO 9660 identifier"));
        }
        
        // Read volume space size (both endian at offset 80)
        let volume_size = u32::from_le_bytes([
            buffer[80], buffer[81], buffer[82], buffer[83]
        ]);
        
        // Read root directory record (at offset 156, 34 bytes)
        let root_dir = self.parse_directory_record(&buffer[156..190])?;
        
        self.volume_descriptor = Some(VolumeDescriptor {
            type_code: buffer[0],
            identifier: [buffer[1], buffer[2], buffer[3], buffer[4], buffer[5]],
            version: buffer[6],
            volume_space_size: volume_size,
            root_directory_record: root_dir,
        });
        
        Ok(())
    }
    
    /// Parse a directory record from bytes
    fn parse_directory_record(&self, data: &[u8]) -> Result<DirectoryRecord> {
        if data.len() < 33 {
            return Err(anyhow::anyhow!("Directory record too short"));
        }
        
        let length = data[0];
        if length == 0 {
            return Err(anyhow::anyhow!("Invalid directory record length"));
        }
        
        // Location of extent (LBA)
        let location = u32::from_le_bytes([
            data[2], data[3], data[4], data[5]
        ]);
        
        // Data length
        let data_length = u32::from_le_bytes([
            data[10], data[11], data[12], data[13]
        ]);
        
        // File flags (bit 1 = directory)
        let flags = data[25];
        let is_directory = (flags & 0x02) != 0;
        
        // File identifier length
        let fi_len = data[32] as usize;
        
        // File identifier
        let file_identifier = if fi_len > 0 && data.len() > 33 {
            let fi_end = (33 + fi_len).min(data.len());
            String::from_utf8_lossy(&data[33..fi_end]).to_string()
        } else {
            String::new()
        };
        
        Ok(DirectoryRecord {
            length,
            location,
            data_length,
            is_directory,
            file_identifier,
        })
    }
    
    /// Extract a directory and its contents
    fn extract_directory(&mut self, dir_record: &DirectoryRecord, base_path: &Path, rel_path: &str) -> Result<()> {
        // Read directory contents
        let sector_offset = dir_record.location as u64 * SECTOR_SIZE as u64;
        self.file.seek(SeekFrom::Start(sector_offset))?;
        
        let mut dir_data = vec![0u8; dir_record.data_length as usize];
        self.file.read_exact(&mut dir_data)?;
        
        let mut offset = 0;
        while offset < dir_data.len() {
            // Check if we've reached padding
            if dir_data[offset] == 0 {
                break;
            }
            
            let record_len = dir_data[offset] as usize;
            if record_len == 0 {
                break;
            }
            
            let record_end = (offset + record_len).min(dir_data.len());
            let record = self.parse_directory_record(&dir_data[offset..record_end])?;
            
            // Skip . and .. entries
            if record.file_identifier == "\0" || record.file_identifier == "\x01" {
                offset += record_len;
                continue;
            }
            
            // Clean up file name (remove version suffix ;1)
            let clean_name = record.file_identifier
                .trim_end_matches(";1")
                .to_string();
            
            let item_path = if rel_path.is_empty() {
                clean_name.clone()
            } else {
                format!("{}/{}", rel_path, clean_name)
            };
            
            let full_path = base_path.join(&clean_name);
            
            if record.is_directory {
                // Create directory and recurse
                fs::create_dir_all(&full_path)?;
                self.extract_directory(&record, base_path, &item_path)?;
            } else {
                // Extract file
                self.extract_file(&record, &full_path)?;
            }
            
            offset += record_len;
        }
        
        Ok(())
    }
    
    /// Extract a single file
    fn extract_file(&mut self, file_record: &DirectoryRecord, output_path: &Path) -> Result<()> {
        // Seek to file data
        let sector_offset = file_record.location as u64 * SECTOR_SIZE as u64;
        self.file.seek(SeekFrom::Start(sector_offset))?;
        
        // Create output file
        let mut output = File::create(output_path)?;
        
        // Copy file data
        let mut remaining = file_record.data_length as usize;
        let mut buffer = vec![0u8; SECTOR_SIZE.min(remaining)];
        
        while remaining > 0 {
            let to_read = buffer.len().min(remaining);
            self.file.read_exact(&mut buffer[..to_read])?;
            io::copy(&mut &buffer[..to_read], &mut output)?;
            remaining -= to_read;
        }
        
        Ok(())
    }
}

/// Simple extraction function for direct use
pub fn extract_iso<P: AsRef<Path>, Q: AsRef<Path>>(iso_path: P, output_dir: Q) -> Result<()> {
    println!("🔓 Extracting ISO with native Rust implementation...");
    
    let mut extractor = IsoExtractor::open(iso_path)?;
    extractor.extract_all(output_dir)?;
    
    println!("✅ ISO extracted successfully!");
    Ok(())
}