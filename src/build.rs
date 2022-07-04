extern crate yaml_rust;
use linked_hash_map::Entry;
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use anyhow::Result;

use std::collections::{HashMap, HashSet, LinkedList};
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

    fs::create_dir_all(format!("{target}/plugins"))?;

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

pub async fn build_configs(includes: &Option<LinkedList<String>>, target: &String) -> Result<()> {
    let mut configs: HashMap<String, Yaml> = HashMap::new();
    let mut ignore_dirs: HashSet<String> = HashSet::new();

    ignore_dirs.insert(target.clone());

    match includes {
        Some(incl) => {
            for include in incl {
                scan_configs_dir(Path::new(include), &mut configs, include, &ignore_dirs)?;
                ignore_dirs.insert(include.clone());
            }
        }
        None => {}
    }

    scan_configs_dir(
        Path::new("./"),
        &mut configs,
        &String::from("./"),
        &ignore_dirs,
    )?;

    for (key, value) in configs {
        let path_str = format!("{target}/{key}");
        let path = Path::new(&path_str);

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&value)?;

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, out_str)?;
    }
    Ok(())
}

fn scan_configs_dir(
    dir: &Path,
    configs: &mut HashMap<String, Yaml>,
    strip_prefix: &String,
    ignore_dirs: &HashSet<String>,
) -> Result<()> {
    if ignore_dirs.contains(&String::from(dir.to_str().unwrap())) {
        return Ok(());
    }

    let files = fs::read_dir(dir)?;
    for file in files {
        let file = file?;
        let path = file.path();

        if path.is_file() {
            let extension = path.extension();
            match extension {
                Some(ext) => {
                    if ext == "yml" {
                        let file_data = fs::read_to_string(&path)?;
                        let file_path_stripped =
                            String::from(path.strip_prefix(strip_prefix)?.to_str().unwrap());
                        handle_config(configs, file_path_stripped, file_data)?;
                    }
                }
                None => {}
            }
        } else if path.is_dir() {
            scan_configs_dir(&path, configs, strip_prefix, &ignore_dirs)?;
        }
    }

    Ok(())
}

fn handle_config(
    configs: &mut HashMap<String, Yaml>,
    config_name: String,
    config_data: String,
) -> Result<()> {
    let parsed = YamlLoader::load_from_str(&config_data)?;
    let parsed = &parsed[0];

    if configs.contains_key(&config_name) {
        let current_config = configs.get(&config_name).unwrap();
        let new_config = merge_yamls(current_config, parsed);
        configs.insert(config_name, new_config);
    } else {
        configs.insert(config_name, parsed.clone());
    };

    Ok(())
}

// Merge two YAMLs
fn merge_yamls(a: &Yaml, b: &Yaml) -> Yaml {
    if let Yaml::Hash(a_hash) = a {
        if let Yaml::Hash(b_hash) = b {
            let c = merge_hashes(a_hash, b_hash);
            Yaml::Hash(c)
        } else {
            todo!("error");
        }
    } else {
        todo!("error");
    }
}

// Merge two YAML hashes
fn merge_hashes(a: &Hash, b: &Hash) -> Hash {
    let mut r = a.clone();
    for (k, v) in b.iter() {
        if let Yaml::Hash(bh) = v {
            if let Entry::Occupied(e) = r.entry(k.clone()) {
                if let Yaml::Hash(mut rh) = e.get().clone() {
                    rh = merge_hashes(&rh, bh);
                    r.insert(k.clone(), Yaml::Hash(rh));
                    continue;
                }
            }
        }
        r.insert(k.clone(), v.clone());
    }
    r
}
