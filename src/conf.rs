//! Configuration of `f`

use crate::traits::Toml;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Conf {
    pub use_human_units: bool,
    pub preview_files: bool,
}

impl Toml for Conf {}
