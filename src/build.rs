use anyhow::Result;

use std::collections::{HashMap, HashSet, LinkedList};
use std::fs;

use crate::{config::Plugin, download};

pub async fn build_plugins(
    plugins: &LinkedList<Plugin>,
    lock: &HashMap<String, String>,
    target: &String,
) -> Result<()> {
    let mut plugin_files: HashSet<String> = HashSet::new();

    for plugin in plugins.iter() {
        plugin_files.insert(format!("{}.jar", plugin.name_version()));

        let plugin_hash = lock.get(&plugin.name);
        let plugin_hash = match plugin_hash {
            Some(ph) => ph,
            None => todo!("invalid lock error"),
        };

        download::save_plugin(&plugin, plugin_hash, target).await?;
    }

    let files_in_plugins_dir = fs::read_dir(format!("{target}/plugins/"))?;
    for file_in_plugins_dir in files_in_plugins_dir {
        let file_name = file_in_plugins_dir?.file_name();
        match file_name.to_str() {
            Some(name) => {
                if name.ends_with(".jar") && !plugin_files.contains(name) {
                    fs::remove_file(format!("{target}/plugins/{name}"))?;
                }
            }
            None => {}
        }
    }

    Ok(())
}
