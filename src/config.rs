use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;

use std::collections::{HashMap, LinkedList};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub include: Option<LinkedList<String>>,
    pub launch: Launch,
    pub core: Core,
    pub plugins: HashMap<String, Plugin>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Launch {
    pub args: LinkedList<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Core {
    pub url: String,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub version: String,
    pub url: Option<String>,
}

pub fn load_config() -> Result<Config> {
    let config_file = fs::read_to_string("./mcstarter.yml")?;
    let config: Config = serde_yaml::from_str(&config_file)?;
    Ok(config)
}
