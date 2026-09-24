use super::ResolvedPackage;
mod documents;
mod impact;
mod importers;
mod resolution;
pub(crate) use documents::{load_documents, package_key_strings, package_maps, PnpmDocuments};
pub(crate) use impact::{impact_importer_paths, impact_names};
pub use importers::{parse_importers, PnpmImporter, PnpmImporterDependency};
pub(crate) use importers::{parse_importers_for_impact, PnpmImpactImporter};
pub fn parse(content: &str) -> Vec<ResolvedPackage> {
    // Malformed input stays an empty inventory. Schema rejection is separate
    // and belongs to `validate_for_planning`.
    let Ok(docs) = documents::load_documents(content) else {
        return Vec::new();
    };
    let mut packages = Vec::new();
    for packages_map in documents::package_maps(&docs) {
        for (key, value) in packages_map {
            let key_str = yaml_key_to_string(key);
            let (name, version) = split_name_version(&key_str);
            let (fingerprint, kind) = resolution::resolution_info(value);
            packages.push(ResolvedPackage {
                name: name.to_string(),
                version: version.to_string(),
                fingerprint,
                kind,
            });
        }
    }
    packages
}

/// Planning needs to distinguish unsafe input from an intentionally empty
/// parse result. Keep the public parser's historical empty-result behavior.
#[derive(Clone, Copy, Debug)]
pub(crate) enum PnpmValidationError {
    Malformed,
    UnsupportedSchema,
    /// A document before the last one is not an env lockfile.
    NotEnvPrefix,
}

pub(crate) fn validation_detail(error: PnpmValidationError) -> &'static str {
    match error {
        PnpmValidationError::Malformed => "could not be parsed",
        PnpmValidationError::NotEnvPrefix => "has a non-env document before the project lockfile",
        PnpmValidationError::UnsupportedSchema => "has an unsupported schema",
    }
}

pub(crate) fn prepare_documents(content: &str) -> Result<PnpmDocuments, PnpmValidationError> {
    let docs = documents::load_documents(content)?;
    documents::validate_supported(&docs)?;
    documents::validate_env_prefix(&docs)?;
    Ok(docs)
}

pub(crate) fn validate_for_planning(content: &str) -> Result<(), PnpmValidationError> {
    prepare_documents(content).map(|_| ())
}

/// Returns changed top-level fields that alter installation behavior but are
/// not represented by the importer/package/snapshot dependency graph.
pub(crate) fn changed_unmodeled_installation_sections(old: &str, new: &str) -> Vec<String> {
    let Ok(old) = documents::load_documents(old) else {
        return Vec::new();
    };
    let Ok(new) = documents::load_documents(new) else {
        return Vec::new();
    };
    let Some(old) = old.project_mapping() else {
        return Vec::new();
    };
    let Some(new) = new.project_mapping() else {
        return Vec::new();
    };
    old.keys()
        .chain(new.keys())
        .filter_map(serde_yaml::Value::as_str)
        .filter(|key| !matches!(*key, "importers" | "packages" | "snapshots"))
        .filter(|key| old.get(*key) != new.get(*key))
        .map(str::to_string)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn split_name_version(key: &str) -> (&str, &str) {
    // Strip peer-dep suffix enclosed in parens: pkg@ver(peer@X) → pkg@ver
    let base = key.split_once('(').map_or(key, |(b, _)| b);
    // Strip pnpm v5/v6 leading slash (e.g. `/lodash@4.17.21` → `lodash@4.17.21`).
    // Track whether the key had a leading slash: only v5 slash-separated keys do,
    // so we use this to distinguish `/lodash/4.17.21` from `github.com/org/repo`.
    let had_leading_slash = base.starts_with('/');
    let base = base.trim_start_matches('/');

    if let Some(stripped) = base.strip_prefix('@') {
        // Scoped package. suffix = "scope/pkg[@ver | /ver]..."
        let Some(first_slash) = stripped.find('/') else {
            return (base, "");
        };
        let first_slash = first_slash + 1; // adjust past leading '@'
        let pkg_rest = &base[first_slash + 1..]; // "pkg@ver" (v6/v7) or "pkg/ver" (v5)
        let at_in_pkg = pkg_rest.find('@');
        let slash_in_pkg = pkg_rest.find('/');
        match (at_in_pkg, slash_in_pkg) {
            (Some(a), Some(s)) if s < a => {
                // v5 scoped with peer suffix: @scope/pkg/ver_peer@ver
                let ver_raw = &pkg_rest[s + 1..];
                (
                    &base[..first_slash + 1 + s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (Some(a), _) => {
                // v6/v7 scoped: @scope/pkg@ver[_peer]
                let ver_raw = &pkg_rest[a + 1..];
                (
                    &base[..first_slash + 1 + a],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, Some(s)) => {
                // v5 scoped without peer suffix: @scope/pkg/ver
                let ver_raw = &pkg_rest[s + 1..];
                (
                    &base[..first_slash + 1 + s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, None) => (base, ""),
        }
    } else {
        // Unscoped. Prefer the first '@' (version sep) over '/' (v5 sep) unless '/' comes first.
        // Only treat '/' as a version separator when the original key had a leading slash;
        // without it, slashes are part of the name (e.g. `github.com/org/repo`).
        let first_at = base.find('@');
        let first_slash = base.find('/');
        match (first_at, first_slash) {
            (Some(a), Some(s)) if had_leading_slash && s < a => {
                // v5 unscoped with peer suffix: /pkg/ver_peer@ver
                let ver_raw = &base[s + 1..];
                (
                    &base[..s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (Some(a), _) => {
                // v6/v7 unscoped: pkg@ver[_peer]
                let ver_raw = &base[a + 1..];
                (
                    &base[..a],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            (None, Some(s)) if had_leading_slash => {
                // v5 unscoped without peer suffix: /pkg/ver
                let ver_raw = &base[s + 1..];
                (
                    &base[..s],
                    ver_raw.split_once('_').map_or(ver_raw, |(v, _)| v),
                )
            }
            _ => (base, ""),
        }
    }
}

pub(super) fn yaml_key_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests;
