fn load_package(
    dir: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Result<Option<WorkspacePackage>> {
    let pkg_path = dir.join("package.json");
    if !workspace_path_is_file(&pkg_path, visible_files) {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&pkg_path)?;
    let pkg: PackageJson = serde_json::from_str(&content).unwrap_or_default();

    let name = match pkg.name {
        Some(ref n) if !n.is_empty() => n.clone(),
        _ => return Ok(None),
    };

    // Resolve the entry file in priority order: exports > module > main > types
    let entry = resolve_entry_with_visibility(dir, &pkg, visible_files);

    Ok(Some(WorkspacePackage {
        name,
        dir: dir.to_path_buf(),
        entry,
        exports: pkg.exports.clone(),
        imports: pkg.imports.clone(),
    }))
}

pub fn load_root_package(dir: &Path) -> Result<Option<WorkspacePackage>> {
    let files = crate::codebase::ts_source::discover_visible_paths(dir);
    load_root_package_from_files(dir, &files)
}

#[doc(hidden)]
pub fn load_root_package_from_files(
    dir: &Path,
    files: &[PathBuf],
) -> Result<Option<WorkspacePackage>> {
    let manifest = normalize_path(&dir.join("package.json"));
    if !files
        .iter()
        .any(|path| normalize_path(path) == manifest)
    {
        return Ok(None);
    }
    let visible: std::collections::HashSet<PathBuf> = files
        .iter()
        .map(|path| normalize_path(path))
        .collect();
    load_package(dir, Some(&visible))
}

fn resolve_entry_with_visibility(
    dir: &Path,
    pkg: &PackageJson,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    // Check `exports` first (supports both string and `{".": ...}` forms).
    if let Some(exports) = &pkg.exports {
        if let Some(entry_str) = exports_to_entry_path(exports) {
            let candidate = normalize_path(&dir.join(entry_str));
            if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
                return Some(resolved);
            }
        }
    }

    // module field (ESM)
    if let Some(module) = &pkg.module {
        let candidate = normalize_path(&dir.join(module));
        if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
            return Some(resolved);
        }
    }

    // main field (CJS/default)
    if let Some(main) = &pkg.main {
        let candidate = normalize_path(&dir.join(main));
        if let Some(resolved) = resolve_workspace_path(&candidate, visible_files) {
            return Some(resolved);
        }
    }

    // types field
    if let Some(types) = &pkg.types {
        let candidate = normalize_path(&dir.join(types));
        if workspace_path_is_file(&candidate, visible_files) {
            return Some(candidate);
        }
    }

    // Fallback: try common entry file names.
    for name in &[
        "src/index.mts",
        "src/index.ts",
        "src/index.tsx",
        "index.mts",
        "index.ts",
    ] {
        let p = normalize_path(&dir.join(name));
        if workspace_path_is_file(&p, visible_files) {
            return Some(p);
        }
    }

    None
}

fn resolve_workspace_path(
    path: &Path,
    visible_files: Option<&std::collections::HashSet<PathBuf>>,
) -> Option<PathBuf> {
    match visible_files {
        Some(visible) => try_resolve_from_visible(path, visible),
        None => try_resolve(path),
    }
}

fn exports_to_entry_path(exports: &serde_json::Value) -> Option<String> {
    match exports {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Object(map) => {
            if let Some(dot) = map.get(".") {
                return exports_to_entry_path(dot);
            }
            ["import", "default", "require", "types"]
                .iter()
                .find_map(|key| map.get(*key).and_then(exports_to_entry_path))
        }
        _ => None,
    }
}

#[inline(never)]
fn resolve_export_subpath(exports: &serde_json::Value, subpath: &str) -> Option<String> {
    let serde_json::Value::Object(map) = exports else {
        return None;
    };

    if let Some(value) = map.get(subpath) {
        return exports_to_entry_path(value);
    }

    let mut patterns = Vec::new();
    for (pattern, value) in map {
        if let Some(star_idx) = pattern.find('*') {
            patterns.push((pattern, value, star_idx));
        }
    }
    patterns.sort_by(compare_export_patterns);

    for (pattern, value, star_idx) in patterns {
        if pattern[star_idx + 1..].contains('*') {
            continue;
        }
        let prefix = &pattern[..star_idx];
        let suffix = &pattern[star_idx + 1..];
        let Some(capture) = subpath
            .strip_prefix(prefix)
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            continue;
        };
        let Some(target) = exports_to_entry_path(value) else {
            continue;
        };
        if target.matches('*').count() == 1 {
            return Some(target.replacen('*', capture, 1));
        }
    }

    None
}

fn compare_export_patterns(
    (a, _, a_star): &(&String, &serde_json::Value, usize),
    (b, _, b_star): &(&String, &serde_json::Value, usize),
) -> Ordering {
    let star_order = b_star.cmp(a_star);
    if star_order != Ordering::Equal {
        return star_order;
    }
    a.cmp(b)
}

#[inline(never)]
fn package_name_and_subpath(specifier: &str) -> Option<(String, Option<String>)> {
    if specifier.starts_with('.') || specifier.starts_with('/') {
        return None;
    }

    let mut parts = specifier.splitn(3, '/');
    let first = parts.next().unwrap_or("");
    if first.starts_with('@') {
        let scope_pkg = parts.next()?;
        let name_len = first.len() + 1 + scope_pkg.len();
        let subpath = specifier
            .get(name_len + 1..)
            .map(|rest| format!("./{rest}"));
        return Some((specifier[..name_len].to_string(), subpath));
    }

    let subpath = specifier
        .get(first.len() + 1..)
        .map(|rest| format!("./{rest}"));
    Some((first.to_string(), subpath))
}
