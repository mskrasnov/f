//! Trait objects

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use toml;

pub trait Toml {
    fn parse<P: AsRef<Path>>(pth: P) -> Result<Self>
    where
        for<'de> Self: Deserialize<'de>,
    {
        let contents = fs::read_to_string(&pth)?;
        let data = toml::from_str(&contents)?;

        Ok(data)
    }

    fn write<P: AsRef<Path>>(&self, pth: P) -> Result<()>
    where
        Self: Serialize + Sized,
    {
        let contents = toml::to_string(&self)?;
        fs::write(&pth, contents)?;

        Ok(())
    }
}
