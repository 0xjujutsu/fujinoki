use std::collections::HashMap;

use anyhow::Result;
use fujinoki_updater::{make_registry_url, FetchDistTags};

const DEFAULT_TAG: &str = "latest";

async fn fetch_dist_tags(name: &str) -> Result<HashMap<String, String>> {
    let url = make_registry_url(name);
    let result: FetchDistTags = reqwest::get(&url).await?.json().await?;
    Ok(result.dist_tags)
}

pub async fn get_latest_version(name: &str, tag: Option<&str>) -> Result<Option<String>> {
    let dist_tags = fetch_dist_tags(name).await?;
    let tag = tag.unwrap_or(DEFAULT_TAG);
    Ok(dist_tags.get(tag).cloned())
}
