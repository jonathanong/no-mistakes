/// A single entry from the `packages` section of a pnpm-lock.yaml file.
pub struct PnpmLockPackage {
    /// The package key as written in pnpm-lock.yaml, e.g. `"lodash@4.17.21"`.
    pub key: String,
    /// How the package is resolved: `"repo"`, `"directory"`, `"tarball"`,
    /// `"commit"`, `"integrity"`, or `""` when there is no resolution information.
    pub resolution_kind: String,
}

/// Parse `packages` from every document in a pnpm-lock.yaml file.
///
/// pnpm 12 may write an env document before the project document. Entries from
/// both are returned. An unparsable file, including a later broken document,
/// returns an empty vec.
pub fn parse_pnpm_lock(content: &str) -> Vec<PnpmLockPackage> {
    let Ok(docs) = crate::codebase::lockfile::pnpm::load_documents(content) else {
        return Vec::new();
    };
    let mut packages = Vec::new();
    for packages_map in crate::codebase::lockfile::pnpm::package_maps(&docs) {
        for (key, value) in packages_map {
            packages.push(PnpmLockPackage {
                key: yaml_value_to_string(key),
                resolution_kind: resolve_kind(value),
            });
        }
    }
    packages
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
