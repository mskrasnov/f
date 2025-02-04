//! Create directories after first start

use crate::{
    consts::{CONF_DIR, RECYCLE_BIN_DIR},
    utils::get_home,
};
use anyhow::{anyhow, Result};
use std::fs;

const DIRS: [&str; 2] = [CONF_DIR, RECYCLE_BIN_DIR];

pub fn create_dirs() -> Result<()> {
    let home = get_home();
    for dir in DIRS {
        let dir = home.join(dir);
        fs::create_dir_all(&dir)
            .map_err(|err| anyhow!("Failed to create '{}': {}", dir.display(), err))?;
    }

    Ok(())
}
