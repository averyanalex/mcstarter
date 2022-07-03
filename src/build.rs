use anyhow::Result;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::config::Plugin;
use crate::lock::get_lock_entry;

pub async fn build_plugins(
    plugins: &HashSet<Plugin>,
    lock: &HashMap<String, String>,
    target: &String,
    cache: &String,
) -> Result<()> {
    let mut plugin_files: HashSet<String> = HashSet::new();

    for plugin in plugins {
        let name_version = plugin.name_version();

        let hash = get_lock_entry(&plugin.name, &lock)?;

        let plugin_filename = format!("{name_version}-{hash}.jar");
        let target_path_str = format!("{target}/plugins/{plugin_filename}");
        let target_path = Path::new(&target_path_str);

        plugin_files.insert(plugin_filename);

        if !target_path.exists() {
            fs::copy(format!("{cache}/{hash}"), target_path)?;
        }
    }

    // Remove old plugins
    let files_in_plugins_dir = fs::read_dir(format!("{target}/plugins"))?;
    for file_in_plugins_dir in files_in_plugins_dir {
        let file_in_target_dir = file_in_plugins_dir?;
        let file_name = file_in_target_dir.file_name();
        match file_name.to_str() {
            Some(name) => {
                if name.ends_with(".jar") && !plugin_files.contains(name) {
                    fs::remove_file(file_in_target_dir.path())?;
                }
            }
            None => {}
        }
    }

    Ok(())
}

pub async fn build_core(
    lock: &HashMap<String, String>,
    target: &String,
    cache: &String,
) -> Result<()> {
    let hash = get_lock_entry(&String::from("core"), &lock)?;

    let core_filename = format!("core-{hash}.jar");
    let target_path_str = format!("{target}/{core_filename}");
    let target_path = Path::new(&target_path_str);

    if !target_path.exists() {
        fs::copy(format!("{cache}/{hash}"), target_path)?;
    }

    // Remove old cores
    let files_in_target_dir = fs::read_dir(target)?;
    for file_in_target_dir in files_in_target_dir {
        let file_in_target_dir = file_in_target_dir?;
        let file_name = file_in_target_dir.file_name();
        match file_name.to_str() {
            Some(name) => {
                if name.ends_with(".jar") && name.starts_with("core-") && name != core_filename {
                    fs::remove_file(file_in_target_dir.path())?;
                }
            }
            None => {}
        }
    }

    Ok(())
}
