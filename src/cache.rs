use anyhow::Result;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::config::{Core, Plugin};
use crate::download::{download_hashed_core, download_hashed_plugin};
use crate::lock::get_lock_entry;

pub async fn cache_core(
    core: &Core,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    let hash = get_lock_entry(&String::from("core"), &lock)?;

    let path_str = format!("{cache_dir}/{hash}");
    let path = Path::new(&path_str);

    if !path.exists() {
        let core_bytes = download_hashed_core(core, &hash).await?;
        fs::write(&path, core_bytes)?;
    }
    Ok(())
}

pub async fn cache_plugins(
    plugins: &HashSet<Plugin>,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    for plugin in plugins.iter() {
        cache_plugin(plugin, lock, cache_dir).await?;
    }
    Ok(())
}

async fn cache_plugin(
    plugin: &Plugin,
    lock: &HashMap<String, String>,
    cache_dir: &String,
) -> Result<()> {
    let hash = get_lock_entry(&plugin.name, &lock)?;

    let path_str = format!("{cache_dir}/{hash}");
    let path = Path::new(&path_str);

    if !path.exists() {
        let plugin_bytes = download_hashed_plugin(plugin, &hash).await?;
        fs::write(path, plugin_bytes)?;
    }
    Ok(())
}
