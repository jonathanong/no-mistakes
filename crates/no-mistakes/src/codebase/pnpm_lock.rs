/// A single entry from the `packages` section of a pnpm-lock.yaml file.
pub struct PnpmLockPackage {
    /// The package key as written in pnpm-lock.yaml, e.g. `"lodash@4.17.21"`.
    pub key: String,
    /// How the package is resolved: `"tarball"`, `"directory"`, `"git"`,
    /// `"integrity"`, or `""` (empty) when there is no resolution information.
    pub resolution_kind: String,
}

/// Parse the `packages` section of a pnpm-lock.yaml v9 file.
///
/// Returns an empty vec if the file cannot be parsed or has no `packages`
/// section.
pub fn parse_pnpm_lock(content: &str) -> Vec<PnpmLockPackage> {
    let Ok(root) = serde_yaml::from_str::<serde_yaml::Value>(content) else {
        return Vec::new();
    };

    let Some(packages_map) = root.get("packages").and_then(|v| v.as_mapping()) else {
        return Vec::new();
    };

    packages_map
        .iter()
        .map(|(key, value)| {
            let key_str = yaml_value_to_string(key);
            let resolution_kind = resolve_kind(value);
            PnpmLockPackage {
                key: key_str,
                resolution_kind,
            }
        })
        .collect()
}

/// Determine `resolution_kind` from a package entry value.
///
/// Priority: `repo` > `directory` > `tarball` > `commit` > `integrity` > `""`
fn resolve_kind(value: &serde_yaml::Value) -> String {
    let Some(resolution) = value.get("resolution") else {
        return String::new();
    };

    // Priority order as specified
    for key in &["repo", "directory", "tarball", "commit"] {
        if resolution.get(key).is_some() {
            return (*key).to_string();
        }
    }

    if resolution.get("integrity").is_some() {
        return "integrity".to_string();
    }

    String::new()
}

fn yaml_value_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests;
