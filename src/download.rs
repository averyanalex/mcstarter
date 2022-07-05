extern crate reqwest;

use anyhow::{anyhow, Result};

use bytes::Bytes;

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

pub async fn download_hashed_plugin(
    name: &String,
    plugin: &Plugin,
    expected_hash: &String,
) -> Result<Bytes> {
    let plugin_bytes = download_plugin(plugin).await?;
    let plugin_hash = hash_bytes(&plugin_bytes);

    if &plugin_hash == expected_hash {
        Ok(plugin_bytes)
    } else {
        Err(anyhow!("plugin {name} has invalid hash"))
    }
}

pub async fn download_hashed_core(core: &Core, expected_hash: &String) -> Result<Bytes> {
    let core_bytes = download_core(core).await?;
    let core_hash = hash_bytes(&core_bytes);

    if &core_hash == expected_hash {
        Ok(core_bytes)
    } else {
        Err(anyhow!("core has invalid hash"))
    }
}
