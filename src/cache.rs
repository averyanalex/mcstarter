use anyhow::Result;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::config::{Config, Core, Plugin};
use crate::download::{download_hashed_core, download_hashed_plugin};
use crate::lock::get_lock_entry;

pub async fn cache_core(
    core: &Core,
    config: &Config,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    let hash = get_lock_entry(&String::from("core"), &lock)?;

    let path_str = format!("{cache_dir}/{hash}");
    let path = Path::new(&path_str);

    if !path.exists() {
        let core_bytes = download_hashed_core(core, config, &hash).await?;
        fs::write(&path, core_bytes)?;
    }
    Ok(())
}

pub async fn cache_plugins(
    plugins: &HashMap<String, Plugin>,
    config: &Config,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    for (name, plugin) in plugins {
        cache_plugin(name, plugin, config, lock, cache_dir).await?;
    }
    Ok(())
}

async fn cache_plugin(
    name: &String,
    plugin: &Plugin,
    config: &Config,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    let hash = get_lock_entry(name, &lock)?;

    let path_str = format!("{cache_dir}/{hash}");
    let path = Path::new(&path_str);

    if !path.exists() {
        let plugin_bytes = download_hashed_plugin(name, plugin, config, &hash).await?;
        fs::write(path, plugin_bytes)?;
    }
    Ok(())
}
