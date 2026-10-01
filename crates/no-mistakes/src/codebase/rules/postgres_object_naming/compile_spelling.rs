use anyhow::{bail, Result};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn compile(
    spelling: &BTreeMap<String, String>,
) -> Result<Vec<(String, String, String)>> {
    let mut seen = BTreeSet::new();
    let mut compiled = Vec::new();
    for (key, value) in spelling {
        if key.trim().is_empty() {
            bail!("postgres-object-naming option spelling: empty key");
        }
        if !super::policy::is_single_word(key) {
            bail!("postgres-object-naming option spelling: key \"{key}\" must be a single word");
        }
        if value.trim().is_empty() {
            bail!("postgres-object-naming option spelling: empty value");
        }
        if key.eq_ignore_ascii_case(value) {
            bail!("postgres-object-naming option spelling: key \"{key}\" equals its value");
        }
        if !seen.insert(key.to_ascii_lowercase()) {
            bail!("postgres-object-naming option spelling: duplicate key {key}");
        }
        if spelling
            .keys()
            .any(|other| other.eq_ignore_ascii_case(value))
        {
            bail!("postgres-object-naming option spelling: value \"{value}\" is also a key");
        }
        compiled.push((key.to_ascii_lowercase(), key.clone(), value.clone()));
    }
    Ok(compiled)
}
