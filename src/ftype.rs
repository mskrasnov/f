//! Detects filetypes

use anyhow::{anyhow, Result};
use std::{
    ffi::OsString,
    fmt::Display,
    fs::{self, DirEntry},
    path::PathBuf,
};

use crate::recycle_bin::*;

/// Some file from current listed directory
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// File name without full path
    pub file_name: OsString,

    /// Full path to the file
    pub path: PathBuf,

    /// File size in bytes
    pub byte_size: u64,

    /// Type of this file
    pub file_type: FileType,

    pub is_hidden: bool,
}

impl FileEntry {
    pub fn from_dir_entry(dir_entry: &DirEntry) -> Result<Self> {
        let meta = dir_entry.metadata().map_err(|err| {
            anyhow!(
                "Failed to get metadata of '{}' file: {}",
                dir_entry.path().display(),
                err,
            )
        })?;
        let file_type = FileType::from_fs_file_type(&meta.file_type());
        let byte_size = meta.len();
        let path = dir_entry.path();
        let file_name = dir_entry.file_name();

        Ok(Self {
            file_name,
            path,
            byte_size,
            file_type,
            is_hidden: dir_entry.file_name().to_string_lossy().starts_with('.'),
        })
    }

    /// Get the human file size from bytes
    pub fn size(&self) -> FileSize {
        FileSize::get_human_size(self.byte_size)
    }

    /// Remove files or directories
    pub fn remove(&self) -> Result<()> {
        if self.path.is_dir() {
            fs::remove_dir_all(&self.path)?;
        } else {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }

    /// Remove file/dir to recycle bin
    pub fn remove_bin(&self) -> Result<()> {
        let entry = RecycleBinEntry::new(&self.path.display());
        entry.safe_delete()
    }
}

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    FileExecutable, // executable ELF or text script
    ParentDirectory,
    Directory,
    Link,    // only symbolic links supported yet
    Special, // special (e.g. block, pipes) or unknown file type
}

impl FileType {
    pub fn from_fs_file_type(ftype: &fs::FileType) -> Self {
        if ftype.is_file() {
            Self::File
        } else if ftype.is_dir() {
            Self::Directory
        } else if ftype.is_symlink() {
            Self::Link
        } else {
            Self::Special
        }
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::File => "file",
                Self::FileExecutable => "file*",
                Self::ParentDirectory => "UP-DIR",
                Self::Directory => "DIR",
                Self::Link => "link",
                Self::Special => "special",
            }
        )
    }
}

pub enum FileSize {
    Bytes(u64),
    KBytes(f64),
    MBytes(f64),
    GBytes(f64),
    TBytes(f64),
}

impl FileSize {
    pub fn get_human_size(byte_size: u64) -> Self {
        let i: u64 = 2;
        if byte_size >= i.pow(40) {
            Self::TBytes(byte_size as f64 / 1024_f64.powf(4.))
        } else if byte_size >= i.pow(30) {
            Self::GBytes(byte_size as f64 / 1024_f64.powf(3.))
        } else if byte_size >= i.pow(20) {
            Self::MBytes(byte_size as f64 / 1024_f64.powf(2.))
        } else if byte_size >= i.pow(10) {
            Self::KBytes(byte_size as f64 / 1024.)
        } else {
            Self::Bytes(byte_size)
        }
    }
}

impl Display for FileSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bytes(len) => format!("{len} Bytes"),
                Self::KBytes(len) => format!("{len:.2} KBytes"),
                Self::MBytes(len) => format!("{len:.2} MBytes"),
                Self::GBytes(len) => format!("{len:.2} GBytes"),
                Self::TBytes(len) => format!("{len:.2} TBytes"),
            }
        )
    }
}
