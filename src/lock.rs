use anyhow::Result;
use std::fs;

use std::collections::HashMap;

// pub fn load_lock() -> Result<HashMap<String, String>> {
//     let lock_file = fs::read_to_string("./mcstarter.lock")?;
//     let lock: HashMap<String, String> = serde_yaml::from_str(&lock_file)?;
//     Ok(lock)
// }

pub fn save_lock(lock: HashMap<String, String>) -> Result<()> {
    let lock = serde_yaml::to_string(&lock)?;
    fs::write("./mcstarter.lock", lock)?;
    Ok(())
}
