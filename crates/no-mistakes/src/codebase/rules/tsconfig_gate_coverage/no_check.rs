//! Prepared-source resolution of effective `compilerOptions.noCheck` values.
//!
//! This rule needs only one compiler option, so it deliberately does not build
//! another tsconfig catalog. It follows local `extends` files through the
//! request's `SourceStore`, preserving one read/cache identity for the check.

use crate::codebase::ts_source::SourceStore;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

/// Return projects whose gate commands cannot prove full typechecking.
///
/// A literal effective `noCheck: true` is non-enforcing. Unresolved configs are
/// left to TypeScript itself: a read or parse failure makes `tsc` fail rather
/// than silently accepting ordinary type errors.
pub(super) fn non_enforcing_tsconfigs(
    root: &Path,
    tracked: &BTreeSet<String>,
    sources: &SourceStore,
) -> BTreeSet<String> {
    tracked
        .iter()
        .filter(|project| {
            matches!(
                effective_no_check(&root.join(project), sources, &mut HashSet::new()),
                Ok(Some(true))
            )
        })
        .cloned()
        .collect()
}

fn effective_no_check(
    path: &Path,
    sources: &SourceStore,
    loading: &mut HashSet<PathBuf>,
) -> Result<Option<bool>, ()> {
    let path = crate::codebase::ts_resolver::normalize_path(path);
    if !loading.insert(path.clone()) {
        return Err(());
    }
    let result = effective_no_check_inner(&path, sources, loading);
    loading.remove(&path);
    result
}

fn effective_no_check_inner(
    path: &Path,
    sources: &SourceStore,
    loading: &mut HashSet<PathBuf>,
) -> Result<Option<bool>, ()> {
    let source = sources.read_path(path).map_err(|_| ())?;
    let parsed: Option<serde_json::Value> =
        jsonc_parser::parse_to_serde_value(&source, &jsonc_parser::ParseOptions::default())
            .map_err(|_| ())?;
    let value = parsed.unwrap_or(serde_json::Value::Null);
    let dir = path.parent().ok_or(())?;
    let mut inherited = None;
    for extends in extends_values(&value)? {
        let extended = resolve_local_extends(dir, &extends, sources)?;
        if let Some(value) = effective_no_check(&extended, sources, loading)? {
            inherited = Some(value);
        }
    }
    Ok(own_no_check(&value)?.or(inherited))
}

fn extends_values(value: &serde_json::Value) -> Result<Vec<String>, ()> {
    match value.get("extends") {
        None => Ok(Vec::new()),
        Some(serde_json::Value::String(path)) => Ok(vec![path.clone()]),
        Some(serde_json::Value::Array(paths)) => paths
            .iter()
            .map(|path| path.as_str().map(ToString::to_string).ok_or(()))
            .collect(),
        Some(_) => Err(()),
    }
}

fn resolve_local_extends(dir: &Path, extends: &str, sources: &SourceStore) -> Result<PathBuf, ()> {
    if !extends.starts_with('.') {
        return Err(());
    }
    let candidate = crate::codebase::ts_resolver::normalize_path(&dir.join(extends));
    if candidate.extension() == Some(std::ffi::OsStr::new("json"))
        || sources
            .inventory()
            .id_for_normalized_path(&candidate)
            .is_some()
    {
        return Ok(candidate);
    }
    let mut file = candidate.as_os_str().to_os_string();
    file.push(".json");
    Ok(PathBuf::from(file))
}

fn own_no_check(value: &serde_json::Value) -> Result<Option<bool>, ()> {
    let Some(compiler_options) = value.get("compilerOptions") else {
        return Ok(None);
    };
    let compiler_options = compiler_options.as_object().ok_or(())?;
    match compiler_options.get("noCheck") {
        None => Ok(None),
        Some(serde_json::Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(()),
    }
}
