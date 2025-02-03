//! Get information about installed Linux-system
//!
//! > **Warning:** only Linux is supported yet!

use anyhow::{anyhow, Result};
use std::{collections::HashMap, fs};

use crate::consts::{OS_RELEASE_FILE, UNAME_FILE};

/// Some data from `/etc/os-release` and `/proc/version` files
#[derive(Debug)]
pub struct OsRelease {
    /// A string identifying the operating system, without a version
    /// component, and suitable for presentation to the user. If not
    /// set, a default of "NAME=UNIX" may be used.
    pub name: String,

    /// A string identifying the operating system version, excluding
    /// any OS name information, possibly including a release code
    /// name, and suitable for presentation to the user. This field
    /// is optional.
    pub version: Option<String>,

    /// A lower-case string (no spaces or other characters outside of
    /// 0–9, a–z, ".", "_" and "-") identifying the operating system,
    /// excluding any version information and suitable for processing
    /// by scripts or usage in generated filenames. If not set, a
    /// default of "ID=unix" may be used. Note that even though this
    /// string may not include characters that require shell quoting,
    /// quoting may nevertheless be used.
    pub id: String,

    /// Data from `/etc/version` file
    pub uname: String,
}

impl OsRelease {
    fn read_os_release() -> Result<String> {
        let data = fs::read_to_string(OS_RELEASE_FILE)?;

        Ok(data)
    }

    fn parse_os_release(contents: &str) -> HashMap<String, String> {
        let contents = contents.split('\n');
        let mut data = HashMap::new();

        for chunk in contents {
            let pair = chunk.trim().split('=').collect::<Vec<_>>();
            if pair.len() != 2 {
                continue;
            }
            data.insert(pair[0].to_string(), pair[1].to_string());
        }

        data
    }

    fn read_uname() -> Result<String> {
        let data = fs::read_to_string(UNAME_FILE)?.trim().to_string();

        Ok(data)
    }

    pub fn parse() -> Result<Self> {
        let os_release = Self::read_os_release()
            .map_err(|err| anyhow!("Failed to read '{}' file: {}", OS_RELEASE_FILE, err))?;
        let os_release = Self::parse_os_release(&os_release);

        let uname = Self::read_uname()
            .map_err(|err| anyhow!("Failed to read '{}' file: {}", UNAME_FILE, err))?;

        Ok(Self {
            name: os_release
                .get("NAME")
                .unwrap_or(&"UNIX".to_string())
                .to_string(),
            version: if let Some(ver) = os_release.get("VERSION") {
                Some(ver.to_string())
            } else {
                None
            },
            id: os_release
                .get("ID")
                .unwrap_or(&"unix".to_string())
                .to_string(),
            uname,
        })
    }
}
