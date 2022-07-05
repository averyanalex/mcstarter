use linked_hash_map::Entry;
use yaml_rust::yaml::Hash;
use yaml_rust::yaml::Yaml;

use anyhow::{anyhow, Result};

// Merge two YAMLs
pub fn merge_yamls(a: &Yaml, b: &Yaml) -> Result<Yaml> {
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
