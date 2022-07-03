extern crate reqwest;

use anyhow::Result;
use bytes::Bytes;

use std::fs;

use crate::config::{Core, Plugin};
use crate::hash::hash_bytes;

async fn download_url(url: &String) -> Result<Bytes> {
    let resp = reqwest::get(url).await?.bytes().await?;
    Ok(resp)
}

pub async fn download_core(core: &Core) -> Result<Bytes> {
    let core_bytes = download_url(&core.url).await?;
    Ok(core_bytes)
}

pub async fn download_plugin(plugin: &Plugin) -> Result<Bytes> {
    match &plugin.url {
        Some(url) => {
            let plugin_bytes = download_url(&url).await?;
            Ok(plugin_bytes)
        }
        None => todo!("download from spiget"),
    }
}

pub async fn download_hashed_plugin(plugin: &Plugin, expected_hash: &String) -> Result<Bytes> {
    let plugin_bytes = download_plugin(plugin).await?;
    let plugin_hash = hash_bytes(&plugin_bytes);

    if &plugin_hash == expected_hash {
        Ok(plugin_bytes)
    } else {
        todo!("return error plugin hash invalid")
    }
}

pub async fn download_hashed_core(core: &Core, expected_hash: &String) -> Result<Bytes> {
    let core_bytes = download_core(core).await?;
    let core_hash = hash_bytes(&core_bytes);

    if &core_hash == expected_hash {
        Ok(core_bytes)
    } else {
        todo!("return error core hash invalid")
    }
}

pub async fn save_plugin(plugin: &Plugin, hash: &String, target: &String) -> Result<()> {
    let name_version = plugin.name_version();
    let save_path = format!("{target}/plugins/{name_version}.jar");

    if !file_matchs_hash(&save_path, &hash) {
        let plugin_bytes = download_hashed_plugin(plugin, &hash).await?;
        fs::create_dir_all(format!("{target}/plugins"))?;
        fs::write(save_path, &plugin_bytes)?;
    }

    Ok(())
}

pub async fn save_core(core: &Core, hash: &String, target: &String) -> Result<()> {
    let save_path = format!("{target}/core.jar");

    if !file_matchs_hash(&save_path, &hash) {
        let core_bytes = download_hashed_core(core, &hash).await?;
        fs::create_dir_all(target)?;
        fs::write(save_path, &core_bytes)?;
    }

    Ok(())
}

fn file_matchs_hash(path: &String, hash: &String) -> bool {
    let file = fs::read(path);
    match file {
        Ok(file_content) => {
            if &hash_bytes(&Bytes::from(file_content)) == hash {
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}
