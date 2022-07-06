use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use std::collections::{HashMap, LinkedList};

use crate::{env, merger};

fn default_includes() -> LinkedList<String> {
    let includes: LinkedList<String> = LinkedList::new();
    includes
}

fn default_sources() -> HashMap<String, Source> {
    let sources: HashMap<String, Source> = HashMap::new();
    sources
}

fn default_plugins() -> HashMap<String, Plugin> {
    let plugins: HashMap<String, Plugin> = HashMap::new();
    plugins
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_includes")]
    pub include: LinkedList<String>,
    #[serde(default = "default_sources")]
    pub sources: HashMap<String, Source>,
    pub default_source: Option<String>,
    pub launch: Launch,
    pub core: Core,
    #[serde(default = "default_plugins")]
    pub plugins: HashMap<String, Plugin>,
}

impl Config {
    pub fn get_default_source(&self) -> Result<&Source> {
        let default_source_name = match &self.default_source {
            Some(default_source) => default_source,
            None => bail!("no default source specified"),
        };
        let source = match self.sources.get(default_source_name) {
            Some(default_source) => default_source,
            None => bail!("source {default_source_name} not found"),
        };
        Ok(source)
    }
}

fn default_java_args() -> LinkedList<String> {
    let java_args: LinkedList<String> = LinkedList::new();
    java_args
}

fn default_mc_args() -> LinkedList<String> {
    let mc_args: LinkedList<String> = LinkedList::new();
    mc_args
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Launch {
    #[serde(default = "default_java_args")]
    pub java_args: LinkedList<String>,
    #[serde(default = "default_mc_args")]
    pub mc_args: LinkedList<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Source {
    pub url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Core {
    pub name: String,
    pub version: String,
    pub source: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Plugin {
    pub version: String,
    pub source: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct IncludesConfig {
    pub include: Option<LinkedList<String>>,
}

pub fn load_config(pass_env: bool) -> Result<Config> {
    let main_config_file = fs::read_to_string("./mcstarter.yml")?;
    let includes_config: IncludesConfig = serde_yaml::from_str(&main_config_file)?;

    match includes_config.include {
        Some(includes) => {
            // TODO: refactor this cringe
            let main_cfg = fs::read_to_string("./mcstarter.yml")?;
            let parsed_main_cfg = &YamlLoader::load_from_str(&main_cfg)?[0];

            let mut yaml_config = Yaml::Null;

            for include in includes {
                let config_path = Path::new(&include).join("mcstarter.yml");
                if config_path.exists() {
                    let config_to_merge = fs::read_to_string(&config_path)?;
                    let parsed_config_to_merge = &YamlLoader::load_from_str(&config_to_merge)?[0];
                    yaml_config = merger::merge_yamls(&yaml_config, parsed_config_to_merge);
                }
            }

            yaml_config = merger::merge_yamls(&yaml_config, parsed_main_cfg);

            let mut config_str = String::new();
            let mut emitter = YamlEmitter::new(&mut config_str);
            emitter.dump(&yaml_config)?;

            let config: Config = if pass_env {
                serde_yaml::from_str(&env::pass_envs(&config_str)?)?
            } else {
                serde_yaml::from_str(&config_str)?
            };

            Ok(config)
        }
        None => {
            let config: Config = serde_yaml::from_str(&main_config_file)?;
            Ok(config)
        }
    }
}
