extern crate reqwest;

use anyhow::{anyhow, bail, Result};

use bytes::Bytes;

use crate::config::{Config, Core, Plugin};
use crate::hash::hash_bytes;

async fn download_url(url: &String) -> Result<Bytes> {
    let resp = reqwest::get(url).await?.bytes().await?;
    Ok(resp)
}

pub async fn download_core(core: &Core, config: &Config) -> Result<Bytes> {
    // URL specified?
    let url = match &core.url {
        // Yes, using it
        Some(url) => url.clone(),

        // No, trying other methods
        // Plugin's source specified?
        None => match &core.source {
            // Yes, using specified source
            Some(source_name) => match config.sources.get(source_name) {
                Some(source) => prepare_url(&source.url, &core.name, &core.version),
                None => bail!("source {source_name} not found"),
            },
            // No, using default source
            None => {
                let source = config.get_default_source()?;
                prepare_url(&source.url, &core.name, &core.version)
            }
        },
    };
    let core_bytes = download_url(&url).await?;
    Ok(core_bytes)
}

fn prepare_url(url: &String, name: &String, version: &String) -> String {
    let mut new_url = url.clone();
    new_url = new_url.replace("$NAME", name);
    new_url = new_url.replace("$VERSION", version);
    new_url
}

pub async fn download_plugin(name: &String, plugin: &Plugin, config: &Config) -> Result<Bytes> {
    // URL specified?
    let url = match &plugin.url {
        // Yes, using it
        Some(url) => url.clone(),

        // No, trying other methods
        // Plugin's source specified?
        None => match &plugin.source {
            // Yes, using specified source
            Some(source_name) => match config.sources.get(source_name) {
                Some(source) => prepare_url(&source.url, name, &plugin.version),
                None => bail!("source {source_name} not found"),
            },
            // No, using default source
            None => {
                let source = config.get_default_source()?;
                prepare_url(&source.url, &name, &plugin.version)
            }
        },
    };
    let plugin_bytes = download_url(&url).await?;
    Ok(plugin_bytes)
}

pub async fn download_hashed_plugin(
    name: &String,
    plugin: &Plugin,
    config: &Config,
    expected_hash: &String,
) -> Result<Bytes> {
    let plugin_bytes = download_plugin(name, plugin, config).await?;
    let plugin_hash = hash_bytes(&plugin_bytes);

    if &plugin_hash == expected_hash {
        Ok(plugin_bytes)
    } else {
        Err(anyhow!("plugin {name} has invalid hash"))
    }
}

pub async fn download_hashed_core(
    core: &Core,
    config: &Config,
    expected_hash: &String,
) -> Result<Bytes> {
    let core_bytes = download_core(core, config).await?;
    let core_hash = hash_bytes(&core_bytes);

    if &core_hash == expected_hash {
        Ok(core_bytes)
    } else {
        Err(anyhow!("core has invalid hash"))
    }
}
