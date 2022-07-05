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
    let mut c = a.clone();
    for (b_k, b_v) in b {
        // If b_v is dict
        if let Yaml::Hash(b_v_hash) = b_v {
            // If b_k exists in c (originally a)
            if let Entry::Occupied(e) = c.entry(b_k.clone()) {
                // If c[b_k] also dict
                if let Yaml::Hash(mut c_hash) = e.get().clone() {
                    c_hash = merge_hashes(&c_hash, b_v_hash);
                    c.insert(b_k.clone(), Yaml::Hash(c_hash));
                    continue;
                }
            }
        }
        if b_v.is_array() {
            if let Entry::Occupied(e) = c.entry(b_k.clone()) {
                let cloned_e = e.get().clone();
                if cloned_e.is_array() {
                    let mut b_v_vector = b_v.as_vec().unwrap().clone();
                    let mut vector = cloned_e.as_vec().unwrap().clone();
                    vector.append(&mut b_v_vector);
                    c.insert(b_k.clone(), Yaml::Array(vector));
                    continue;
                }
            }
        }
        c.insert(b_k.clone(), b_v.clone());
    }
    c
}
