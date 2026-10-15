use super::targets::{Package, Target};
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn parse(
    directory: &Path,
    value: &toml::Value,
    visible: &BTreeSet<&Path>,
    edition: Option<&str>,
) -> Option<Package> {
    let package = value.get("package")?;
    let name = package.get("name")?.as_str()?.to_string();
    let mut targets = Vec::new();
    if enabled(package, "autolib") || value.get("lib").is_some() {
        let lib = value.get("lib");
        {
            let path = lib
                .and_then(|lib| lib.get("path"))
                .and_then(toml::Value::as_str)
                .unwrap_or("src/lib.rs");
            push(
                &mut targets,
                directory,
                visible,
                &name,
                path,
                "--lib",
                lib.is_none_or(|lib| enabled(lib, "test")),
            );
        }
    }
    for (table, auto, base, flag) in [
        ("bin", "autobins", "src/bin", "--bin"),
        ("test", "autotests", "tests", "--test"),
    ] {
        let mut explicit = BTreeSet::new();
        let mut explicit_paths = BTreeSet::new();
        for entry in value
            .get(table)
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
        {
            let target_name = entry.get("name").and_then(toml::Value::as_str)?;
            explicit.insert(target_name.to_string());
            let default = if table == "bin"
                && target_name == name
                && visible.contains(directory.join("src/main.rs").as_path())
            {
                "src/main.rs".to_string()
            } else {
                format!("{base}/{target_name}.rs")
            };
            let path = entry
                .get("path")
                .and_then(toml::Value::as_str)
                .unwrap_or(&default);
            explicit_paths.insert(crate::codebase::ts_resolver::normalize_path(
                &directory.join(path),
            ));
            if !push(
                &mut targets,
                directory,
                visible,
                target_name,
                path,
                flag,
                test_enabled(entry),
            ) && entry.get("path").is_none()
            {
                push(
                    &mut targets,
                    directory,
                    visible,
                    target_name,
                    &format!("{base}/{target_name}/main.rs"),
                    flag,
                    test_enabled(entry),
                );
            }
        }
        if !autodiscover(value, auto, table, edition) {
            continue;
        }
        if table == "bin"
            && !explicit.contains(&name)
            && !explicit_paths.contains(&directory.join("src/main.rs"))
        {
            push(
                &mut targets,
                directory,
                visible,
                &name,
                "src/main.rs",
                flag,
                true,
            );
        }
        let prefix = directory.join(base);
        for path in visible.range::<Path, _>((
            std::ops::Bound::Included(prefix.as_path()),
            std::ops::Bound::Unbounded,
        )) {
            let Ok(relative) = path.strip_prefix(&prefix) else {
                break;
            };
            let parts = relative.components().collect::<Vec<_>>();
            let target_name = match parts.as_slice() {
                [_] if path.extension().is_some_and(|ext| ext == "rs") => {
                    path.file_stem()?.to_string_lossy().into_owned()
                }
                [folder, _] if path.file_name().is_some_and(|file| file == "main.rs") => {
                    folder.as_os_str().to_string_lossy().into_owned()
                }
                _ => continue,
            };
            if !explicit.contains(&target_name) && !explicit_paths.contains(*path) {
                targets.push(Target {
                    name: target_name,
                    entry: path.to_path_buf(),
                    flag,
                    enabled: true,
                });
            }
        }
    }
    Some(Package::new(name, targets, directory))
}

fn autodiscover(manifest: &toml::Value, field: &str, table: &str, edition: Option<&str>) -> bool {
    manifest["package"]
        .get(field)
        .and_then(toml::Value::as_bool)
        .unwrap_or_else(|| {
            edition.is_some_and(|edition| edition != "2015") || manifest.get(table).is_none()
        })
}

fn test_enabled(value: &toml::Value) -> bool {
    enabled(value, "test")
        && value
            .get("required-features")
            .and_then(toml::Value::as_array)
            .is_none_or(Vec::is_empty)
}
fn enabled(value: &toml::Value, field: &str) -> bool {
    value.get(field).and_then(toml::Value::as_bool) != Some(false)
}
fn push(
    targets: &mut Vec<Target>,
    directory: &Path,
    visible: &BTreeSet<&Path>,
    name: &str,
    path: &str,
    flag: &'static str,
    enabled: bool,
) -> bool {
    let entry = crate::codebase::ts_resolver::normalize_path(&directory.join(path));
    if !visible.contains(entry.as_path()) {
        return false;
    }
    targets.push(Target {
        name: name.into(),
        entry,
        flag,
        enabled,
    });
    true
}
