use once_cell::sync::Lazy;
use regex::Regex;

use anyhow::{bail, Result};

use std::env::var;

static ENV_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?:\$\{)([A-Z0-9_]+)(?:\})").unwrap());
static ENV_REGEX_CLEANER: Lazy<Regex> = Lazy::new(|| Regex::new(r"[A-Z0-9_]+").unwrap());

pub fn pass_envs(data: &String) -> Result<String> {
    let mut new_data = data.clone();

    for regex_match in ENV_REGEX.find_iter(data) {
        let regex_match_str = regex_match.as_str();

        let env_name = ENV_REGEX_CLEANER.find(regex_match_str).unwrap().as_str();
        let value = var(env_name);

        match value {
            Ok(value) => {
                new_data = new_data.replace(regex_match_str, &value);
            }
            Err(_) => {
                bail!("environment variable {} not found", env_name)
            }
        }
    }
    Ok(new_data)
}
