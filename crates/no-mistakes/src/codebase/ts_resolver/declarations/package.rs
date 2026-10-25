//! Narrow declaration-only lookup for supplemental ancestry facts.
use crate::codebase::ts_source::{JsonLoadError, SourceStore};
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

#[path = "package_conditions.rs"]
mod conditions;
#[path = "package_context.rs"]
mod context;
#[path = "package_exports.rs"]
mod exports;
#[path = "package_paths.rs"]
mod paths;

pub(super) fn resolve_in_context(
    specifier: &str,
    importer: &Path,
    consumer_root: &Path,
    sources: &SourceStore,
) -> Option<PathBuf> {
    let importer = lookup_context(specifier, importer, consumer_root)?;
    let consumer_root = std::fs::canonicalize(consumer_root).ok()?;
    if specifier.starts_with("./") || specifier.starts_with("../") {
        return paths::declaration(&importer.parent()?.join(specifier), sources).ok()?;
    }
    let (name, subpath, types_name) = package_name(specifier)?;
    let directories: Vec<_> = importer
        .parent()?
        .ancestors()
        .map(|parent| parent.join("node_modules"))
        .collect();
    for modules in &directories {
        let root = modules.join(&name);
        if !installed(&root).ok()? {
            continue;
        }
        if let Some(path) = entry(&root, &subpath, &consumer_root, sources).ok()? {
            return Some(path);
        }
        // An installed nearer package shadows outer versions, even without types.
        break;
    }
    if name.starts_with("@types/") {
        return None;
    }
    for modules in directories {
        let root = modules.join("@types").join(&types_name);
        if installed(&root).ok()? {
            return entry(&root, &subpath, &consumer_root, sources).ok()?;
        }
    }
    None
}

fn installed(path: &Path) -> Result<bool, ()> {
    match std::fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_dir()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(()),
    }
}

fn entry(
    root: &Path,
    subpath: &str,
    consumer_root: &Path,
    sources: &SourceStore,
) -> Result<Option<PathBuf>, ()> {
    // parse_json_path shares both the read_path result and JSON parse slot.
    let manifest = match sources.parse_json_path(&root.join("package.json")) {
        Ok(manifest) => manifest,
        Err(JsonLoadError::Io(error)) if error.kind() == ErrorKind::NotFound => {
            return default_entry(root, subpath, sources);
        }
        Err(_) => return Err(()),
    };
    let object = manifest.as_object().ok_or(())?;
    // Version-dependent type maps need a compiler-version projection we do not own.
    if !object.contains_key("exports") && object.contains_key("typesVersions") {
        return Err(());
    }
    if let Some(value) = object.get("exports") {
        let (typed, targets) = exports::targets(value, subpath, consumer_root, sources)?;
        let mut resolved = None;
        let mut absent = false;
        let mut known = false;
        for target in targets {
            let path = contained(root, &target.path, true)?;
            let found = paths::declaration(&path, sources)?;
            if target.required && found.is_none() {
                return Err(());
            }
            match found {
                Some(path) => {
                    known |= target.known;
                    if resolved.as_ref().is_some_and(|prior| prior != &path) {
                        return Err(());
                    }
                    resolved = Some(path);
                }
                None => absent = true,
            }
        }
        // Missing runtime sidecars cannot defeat an explicit types declaration.
        // Without types, mixed absence/presence makes @types fallback ambiguous.
        if resolved.is_some() && (!known || (!typed && absent)) {
            return Err(());
        }
        return Ok(resolved);
    }
    if subpath.is_empty() {
        if let Some(value) = object.get("types").or_else(|| object.get("typings")) {
            return required(&contained(root, value.as_str().ok_or(())?, false)?, sources);
        }
    }
    if subpath.is_empty() {
        if let Some(main) = object.get("main") {
            let main = contained(root, main.as_str().ok_or(())?, false)?;
            if !matches!(
                main.extension().and_then(|extension| extension.to_str()),
                Some("js" | "mjs" | "cjs")
            ) {
                return Err(());
            }
            return paths::declaration(&main, sources);
        }
    }
    default_entry(root, subpath, sources)
}

fn default_entry(root: &Path, subpath: &str, sources: &SourceStore) -> Result<Option<PathBuf>, ()> {
    let target = if subpath.is_empty() {
        root.join("index")
    } else {
        root.join(subpath)
    };
    paths::declaration(&target, sources)
}

fn contained(root: &Path, target: &str, export: bool) -> Result<PathBuf, ()> {
    if target.is_empty() || target.contains(['\\', '*']) || (export && !target.starts_with("./")) {
        return Err(());
    }
    let path = Path::new(target);
    if path.components().any(|part| {
        matches!(
            part,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(());
    }
    Ok(root.join(path))
}

fn package_name(specifier: &str) -> Option<(String, String, String)> {
    if specifier.contains(['\\', ':', '#', '?']) {
        return None;
    }
    let parts: Vec<_> = specifier.split('/').collect();
    if parts
        .iter()
        .any(|part| part.is_empty() || matches!(*part, "." | ".."))
    {
        return None;
    }
    let count = if specifier.starts_with('@') { 2 } else { 1 };
    if parts.len() < count || parts.first() == Some(&"@") {
        return None;
    }
    let name = parts[..count].join("/");
    let types_name = if count == 2 {
        format!("{}__{}", parts[0].strip_prefix('@')?, parts[1])
    } else {
        name.clone()
    };
    Some((name, parts[count..].join("/"), types_name))
}

fn required(path: &Path, sources: &SourceStore) -> Result<Option<PathBuf>, ()> {
    paths::declaration(path, sources)?.map(Some).ok_or(())
}

#[cfg(test)]
#[path = "package_tests.rs"]
mod tests;

pub(super) fn lookup_context(
    specifier: &str,
    importer: &Path,
    consumer_root: &Path,
) -> Option<PathBuf> {
    context::lookup_importer(specifier, importer, consumer_root)
}
