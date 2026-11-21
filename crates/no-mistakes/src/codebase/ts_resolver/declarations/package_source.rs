use super::*;
use serde_json::Value;

pub(crate) fn resolve_source(
    specifier: &str,
    importer: &Path,
    consumer_root: &Path,
    sources: &SourceStore,
) -> Option<PathBuf> {
    let (name, subpath, _) = package_name(specifier)?;
    let importer = lookup_context(specifier, importer, consumer_root)?;
    for directory in importer.parent()?.ancestors() {
        let root = directory.join("node_modules").join(&name);
        if !installed(&root).ok()? {
            continue;
        }
        let manifest = sources.parse_json_path(&root.join("package.json")).ok()?;
        manifest.get("exports")?;
        if let Some(path) = entry(
            &root,
            &subpath,
            consumer_root,
            mode::ConditionMode::for_importer(&importer),
            sources,
        )
        .ok()
        .flatten()
        {
            return Some(path);
        }
        let targets = exports::source_targets(
            manifest.get("exports")?,
            &subpath,
            mode::ConditionMode::for_importer(&importer),
        )
        .ok()?;
        let mut found = None;
        for target in targets {
            if !target.known {
                return None;
            }
            let path = contained(&root, &target.path, true).ok()?;
            SourceStore::read_prepared_or_open(Some(sources), &path).ok()?;
            if found.as_ref().is_some_and(|prior| prior != &path) {
                return None;
            }
            found = Some(path);
        }
        return found;
    }
    None
}

/// `package_relative` is a slash path under the package root, such as `shape.ts`.
pub(crate) fn matches_runtime_export(exports: &Value, package_relative: &str) -> bool {
    if package_relative.is_empty()
        || package_relative.starts_with(['/', '\\'])
        || package_relative
            .split(['/', '\\'])
            .any(|part| part.is_empty() || part == "..")
    {
        return false;
    }
    let Some(map) = exports.as_object() else {
        return false;
    };
    let file = format!("./{}", package_relative.replace('\\', "/"));
    if map.keys().any(|key| key.starts_with('.')) {
        return map.iter().any(|(key, value)| {
            key.starts_with('.') && key_names_file(exports, key, value, &file)
        });
    }
    map.iter()
        .any(|(key, value)| is_runtime_condition(key) && condition_names_file(value, &file))
}

fn is_runtime_condition(key: &str) -> bool {
    matches!(key, "import" | "default" | "module" | "require")
}

fn condition_names_file(value: &Value, file: &str) -> bool {
    let mut targets = Vec::new();
    collect_strings(value, &mut targets);
    targets.iter().any(|target| same_export_path(target, file))
}

fn key_names_file(exports: &Value, key: &str, value: &Value, file: &str) -> bool {
    if key.contains('*') {
        let mut templates = Vec::new();
        collect_strings(value, &mut templates);
        return templates.into_iter().any(|template| {
            exports::star_subpath(key, &template, file)
                .is_some_and(|subpath| targets_name_file(exports, &subpath, file))
        });
    }
    let subpath = if key == "." {
        ""
    } else {
        key.strip_prefix("./").unwrap_or(key)
    };
    targets_name_file(exports, subpath, file)
}

fn targets_name_file(exports: &Value, subpath: &str, file: &str) -> bool {
    let Ok(targets) = exports::source_targets(exports, subpath, mode::ConditionMode::Import) else {
        return false;
    };
    targets
        .iter()
        .any(|target| target.known && same_export_path(&target.path, file))
}

fn same_export_path(target: &str, file: &str) -> bool {
    let target = target
        .strip_prefix("./")
        .unwrap_or(target)
        .replace('\\', "/");
    let file = file.strip_prefix("./").unwrap_or(file);
    !target.contains('*') && target == file
}

fn collect_strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(text) => out.push(text.clone()),
        Value::Object(map) => {
            for child in map.values() {
                collect_strings(child, out);
            }
        }
        _ => {}
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
#[path = "package_source_match_tests.rs"]
mod match_tests;
