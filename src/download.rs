extern crate reqwest;

use anyhow::Result;
use bytes::Bytes;

pub async fn download(url: &String) -> Result<Bytes> {
    let resp = reqwest::get(url).await?.bytes().await?;
    Ok(resp)
}
