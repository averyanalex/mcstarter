use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use std::collections::LinkedList;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub core: Core,
    pub plugins: LinkedList<Plugin>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Core {
    pub url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub url: String,
}

pub fn load_config() -> Result<Config> {
    let config_file = fs::read_to_string("./mcstarter.yml")?;
    let config: Config = serde_yaml::from_str(&config_file)?;
    Ok(config)
}
