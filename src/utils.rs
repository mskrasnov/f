//! Some utilities and helpers

use std::env::var;
use std::path::{Path, PathBuf};

/// Get path to the user home directory
pub fn get_home() -> PathBuf {
    Path::new(&var("HOME").unwrap_or("/tmp".to_string())).to_path_buf()
}
