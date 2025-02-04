//! Trait objects

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use toml;

pub trait Toml {
    fn parse<P: AsRef<Path>>(pth: P) -> Result<Self>
    where
        for<'de> Self: Deserialize<'de>,
    {
        let contents = fs::read_to_string(&pth)
            .map_err(|err| anyhow!("Failed to read {}: {}", pth.as_ref().display(), err))?;
        let data = toml::from_str(&contents)
            .map_err(|err| anyhow!("Failed to parse {}: {}", pth.as_ref().display(), err))?;

        Ok(data)
    }

    fn write<P: AsRef<Path>>(&self, pth: P) -> Result<()>
    where
        Self: Serialize + Sized,
    {
        let contents =
            toml::to_string(&self).map_err(|err| anyhow!("Failed to deserialize struct: {err}"))?;
        fs::write(&pth, contents)
            .map_err(|err| anyhow!("Failed to write {}: {}", pth.as_ref().display(), err))?;

        Ok(())
    }
}
