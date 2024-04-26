use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub retention: RetentionPolicy,
    pub paths: Vec<RetentionPath>,
}

impl Config {
    pub fn read(path: impl AsRef<Path>) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut file_buffer = String::new();
        file.read_to_string(&mut file_buffer)?;

        let config = toml::from_str(&file_buffer)?;

        Ok(config)
    }
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct RetentionPath {
    pub path: PathBuf,
    pub file_pattern: RetentionFilePattern,
    pub retention: Option<RetentionPolicy>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct RetentionFilePattern(pub String);

#[derive(Default, Debug, Clone, Copy, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct RetentionPolicy {
    pub keep_last: Option<usize>,
    pub keep_hourly: Option<usize>,
    pub keep_daily: Option<usize>,
    pub keep_weekly: Option<usize>,
    pub keep_monthly: Option<usize>,
    pub keep_yearly: Option<usize>,
}
