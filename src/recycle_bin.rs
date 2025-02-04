//! Safe deleting files in recycle bin

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use uuid::Uuid;

use crate::{
    consts::{RECYCLE_BIN_DIR, RECYCLE_BIN_META},
    traits::Toml,
    utils::get_home,
};

#[derive(Deserialize, Serialize, Default)]
pub struct RecycleBin {
    #[serde(rename = "entry")]
    pub entryes: Vec<RecycleBinEntry>,
}

impl Toml for RecycleBin {}

#[derive(Deserialize, Serialize, Clone)]
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
        let rbin_meta = get_home().join(RECYCLE_BIN_META);

        let del_file = home.join(&self.deleted_name);
        let del_file_parent = Path::new(&self.orig_path).parent().unwrap();

        if del_file_parent == &home {
            return self.remove_permanently_orig();
        }

        fs::rename(&self.orig_path, &del_file).map_err(|err| {
            anyhow!(
                "Failed to safe remove the '{}' file: {}",
                &self.orig_path,
                err,
            )
        })?;

        let mut rbin = RecycleBin::parse(&rbin_meta).unwrap_or_default();
        rbin.entryes.push(self.clone());
        rbin.write(&rbin_meta).unwrap();

        Ok(())
    }

    fn remove_permanently_orig(&self) -> Result<()> {
        let pth = Path::new(&self.orig_path);
        if pth.is_dir() {
            fs::remove_dir_all(&pth)
        } else {
            fs::remove_file(&pth)
        }
        .map_err(|err| anyhow!("Failed to remove '{}': {}", pth.display(), err))?;

        Ok(())
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
