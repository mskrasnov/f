//! Safe deleting files in recycle bin

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

use crate::{consts::RECYCLE_BIN_DIR, traits::Toml, utils::get_home};

#[derive(Deserialize, Serialize)]
pub struct RecycleBin {
    #[serde(rename = "entry")]
    pub entryes: Vec<RecycleBinEntry>,
}

impl Toml for RecycleBin {}

#[derive(Deserialize, Serialize)]
pub struct RecycleBinEntry {
    pub orig_path: String,
    pub deleted_name: String, // name with UUID
}

impl RecycleBinEntry {
    pub fn new<T: ToString>(orig_pth: T) -> Self {
        Self {
            orig_path: orig_pth.to_string(),
            deleted_name: {
                let uuid = Uuid::new_v4().simple().to_string();
                uuid // simple UUID without `-`
            },
        }
    }

    pub fn safe_delete(&self) -> Result<()> {
        let home = get_home().join(RECYCLE_BIN_DIR);
        let del_file = home.join(&self.deleted_name);

        fs::rename(&self.orig_path, &del_file).map_err(|err| {
            anyhow!(
                "Failed to safe remove the '{}' file: {}",
                &self.orig_path,
                err,
            )
        })
    }

    pub fn remove_permanently(&self) -> Result<()> {
        let del_file = get_home().join(RECYCLE_BIN_DIR).join(&self.deleted_name);

        if del_file.is_dir() {
            fs::remove_dir_all(&del_file)
        } else {
            fs::remove_file(&del_file)
        }
        .map_err(|err| {
            anyhow!(
                "Failed to permanently remove the '{}' file: {}",
                &self.orig_path,
                err
            )
        })
    }
}
