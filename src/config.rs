use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use yaml_rust::{YamlEmitter, YamlLoader};

use std::collections::{HashMap, LinkedList};

use crate::merger;

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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IncludesConfig {
    pub include: Option<LinkedList<String>>,
}

pub fn load_config() -> Result<Config> {
    let main_config_file = fs::read_to_string("./mcstarter.yml")?;
    let includes_config: IncludesConfig = serde_yaml::from_str(&main_config_file)?;

    match includes_config.include {
        Some(includes) => {
            // TODO: refactor this cringe
            let main_cfg = fs::read_to_string("./mcstarter.yml")?;
            let parsed_main_cfg = &YamlLoader::load_from_str(&main_cfg)?[0];

            let mut yaml_config = parsed_main_cfg.clone();

            for include in includes {
                let config_path = Path::new(&include).join("mcstarter.yml");
                if config_path.exists() {
                    let config_to_merge = fs::read_to_string(&config_path)?;
                    let parsed_config_to_merge = &YamlLoader::load_from_str(&config_to_merge)?[0];
                    yaml_config = merger::merge_yamls(&yaml_config, parsed_config_to_merge)?;
                }
            }

            yaml_config = merger::merge_yamls(&yaml_config, parsed_main_cfg)?;

            let mut config_str = String::new();
            let mut emitter = YamlEmitter::new(&mut config_str);
            emitter.dump(&yaml_config)?;

            let config: Config = serde_yaml::from_str(&config_str)?;

            Ok(config)
        }
        None => {
            let config: Config = serde_yaml::from_str(&main_config_file)?;
            Ok(config)
        }
    }
}
