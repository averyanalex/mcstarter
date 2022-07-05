extern crate yaml_rust;
use linked_hash_map::Entry;
use yaml_rust::yaml::Hash;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

use anyhow::{anyhow, Result};

use std::collections::{HashMap, HashSet, LinkedList};
use std::fs;
use std::path::Path;

use crate::config::Plugin;
use crate::env;
use crate::lock::get_lock_entry;

pub async fn build_plugins(
    plugins: &HashMap<String, Plugin>,
    lock: &HashMap<String, String>,
    target: &String,
    cache: &String,
) -> Result<()> {
    let mut plugin_files: HashSet<String> = HashSet::new();

    fs::create_dir_all(format!("{target}/plugins"))?;

    for (name, plugin) in plugins {
        let name_version = format!("{name}-{}", plugin.version);

        let hash = get_lock_entry(name, &lock)?;

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

pub async fn build_files(includes: &Option<LinkedList<String>>, target: &String) -> Result<()> {
    let mut yml_configs: HashMap<String, Yaml> = HashMap::new();
    let mut etc_configs: HashMap<String, String> = HashMap::new();
    let mut etc_files: HashMap<String, String> = HashMap::new();

    let mut ignore_dirs: HashSet<String> = HashSet::new();

    ignore_dirs.insert(target.clone());
    ignore_dirs.insert(String::from("./.git"));

    match includes {
        Some(incl) => {
            for include in incl {
                scan_dir(
                    Path::new(include),
                    &mut yml_configs,
                    &mut etc_configs,
                    &mut etc_files,
                    include,
                    &ignore_dirs,
                )?;
                ignore_dirs.insert(include.clone());
            }
        }
        None => {}
    }

    scan_dir(
        Path::new("./"),
        &mut yml_configs,
        &mut etc_configs,
        &mut etc_files,
        &String::from("./"),
        &ignore_dirs,
    )?;

    for (key, value) in yml_configs {
        let path_str = format!("{target}/{key}");
        let path = Path::new(&path_str);

        let mut out_str = String::new();
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(&value)?;

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, env::pass_envs(&out_str)?)?;
    }

    for (key, value) in etc_configs {
        let out_path_str = format!("{target}/{key}");
        let out_path = Path::new(&out_path_str);
        let in_path = Path::new(&value);

        let data = fs::read_to_string(&in_path)?;

        fs::create_dir_all(out_path.parent().unwrap())?;
        fs::write(out_path, env::pass_envs(&data)?)?;
    }

    for (key, value) in etc_files {
        let out_path_str = format!("{target}/{key}");
        let out_path = Path::new(&out_path_str);
        let in_path = Path::new(&value);

        fs::create_dir_all(out_path.parent().unwrap())?;

        fs::copy(&in_path, &out_path)?;
    }
    Ok(())
}

fn scan_dir(
    dir: &Path,
    yml_configs: &mut HashMap<String, Yaml>,
    etc_configs: &mut HashMap<String, String>,
    etc_files: &mut HashMap<String, String>,
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
            let file_path_stripped =
                String::from(path.strip_prefix(strip_prefix)?.to_str().unwrap());
            match extension {
                Some(ext) => {
                    if ext == "yml" || ext == "yaml" {
                        handle_yml_config(yml_configs, &file_path_stripped, &path)?;
                    } else if ext == "properties"
                        || ext == "conf"
                        || ext == "txt"
                        || ext == "json"
                        || ext == "toml"
                    {
                        handle_etc_config(etc_configs, &file_path_stripped, &path)?;
                    } else {
                        handle_etc_file(etc_files, &file_path_stripped, &path)?;
                    }
                }
                None => {
                    handle_etc_file(etc_files, &file_path_stripped, &path)?;
                }
            }
        } else if path.is_dir() {
            scan_dir(
                &path,
                yml_configs,
                etc_configs,
                etc_files,
                strip_prefix,
                &ignore_dirs,
            )?;
        }
    }

    Ok(())
}

fn handle_etc_config(
    etc_configs: &mut HashMap<String, String>,
    name: &String,
    file: &Path,
) -> Result<()> {
    etc_configs.insert(name.clone(), String::from(file.to_str().unwrap()));
    Ok(())
}

fn handle_etc_file(
    etc_files: &mut HashMap<String, String>,
    name: &String,
    file: &Path,
) -> Result<()> {
    etc_files.insert(name.clone(), String::from(file.to_str().unwrap()));
    Ok(())
}

fn handle_yml_config(
    yml_configs: &mut HashMap<String, Yaml>,
    name: &String,
    path: &Path,
) -> Result<()> {
    let data = fs::read_to_string(&path)?;
    let parsed = YamlLoader::load_from_str(&data)?;

    if parsed.len() == 0 {
        if !yml_configs.contains_key(name) {
            yml_configs.insert(name.clone(), Yaml::Null);
        }
        Ok(())
    } else if parsed.len() > 1 {
        Err(anyhow!("yaml {} has more than 1 docs", name))
    } else {
        let parsed = &parsed[0];

        let current_config = yml_configs.get(name);

        match current_config {
            Some(current_config) => {
                let new_config = merge_yamls(current_config, parsed)?;
                yml_configs.insert(name.clone(), new_config);
            }
            None => {
                yml_configs.insert(name.clone(), parsed.clone());
            }
        }
        Ok(())
    }
}

// Merge two YAMLs
fn merge_yamls(a: &Yaml, b: &Yaml) -> Result<Yaml> {
    if let Yaml::Hash(a_hash) = a {
        if let Yaml::Hash(b_hash) = b {
            let c = merge_hashes(a_hash, b_hash);
            Ok(Yaml::Hash(c))
        } else {
            Err(anyhow!("can't get yaml's hash"))
        }
    } else {
        Err(anyhow!("can't get yaml's hash"))
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
