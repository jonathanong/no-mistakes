use super::{conditions, mode::ConditionMode};
use serde_json::Value;
use std::cell::OnceCell;
use std::path::Path;

pub(super) struct Target {
    pub(super) path: String,
    pub(super) required: bool,
    pub(super) known: bool,
}

/// Shared JSON maps lack order: every applicable typed branch must agree.
pub(super) fn targets(
    exports: &Value,
    subpath: &str,
    consumer_root: &Path,
    mode: ConditionMode,
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<(bool, Vec<Target>), ()> {
    let selected = if let Some(map) = exports.as_object() {
        if map.keys().any(|key| key.starts_with('.')) {
            if map.keys().any(|key| !key.starts_with('.')) {
                return Err(());
            }
            let key = if subpath.is_empty() {
                ".".to_string()
            } else {
                format!("./{subpath}")
            };
            map.get(&key).ok_or(())?
        } else {
            if !subpath.is_empty() {
                return Err(());
            }
            exports
        }
    } else {
        if !subpath.is_empty() {
            return Err(());
        }
        exports
    };
    if !has_types(selected) {
        let mut targets = Vec::new();
        runtime_targets(selected, true, mode, &mut targets)?;
        if targets.is_empty() {
            return Err(());
        }
        return Ok((false, targets));
    }
    let version = OnceCell::new();
    let compiler = || *version.get_or_init(|| conditions::compiler_version(consumer_root, sources));
    let mut targets = Vec::new();
    select(selected, true, true, mode, &compiler, &mut targets)?;
    if targets.is_empty() {
        return Err(());
    }
    Ok((true, targets))
}

fn has_types(value: &Value) -> bool {
    value.as_object().is_some_and(|map| {
        map.iter()
            .any(|(key, value)| key == "types" || key.starts_with("types@") || has_types(value))
    })
}

fn runtime_targets(
    value: &Value,
    known: bool,
    mode: ConditionMode,
    targets: &mut Vec<Target>,
) -> Result<(), ()> {
    if let Some(text) = value.as_str() {
        let mut target = declaration_target(text, true)?;
        target.known = known;
        targets.push(target);
        return Ok(());
    }
    let map = value.as_object().filter(|map| !map.is_empty()).ok_or(())?;
    for (key, value) in map {
        if key.starts_with('.') {
            return Err(());
        }
        let applies = mode.applies(key);
        if applies == Some(false) {
            continue;
        }
        runtime_targets(value, known && applies == Some(true), mode, targets)?;
    }
    Ok(())
}

fn select(
    value: &Value,
    runtime: bool,
    known: bool,
    mode: ConditionMode,
    compiler: &impl Fn() -> Option<(u64, u64)>,
    targets: &mut Vec<Target>,
) -> Result<(), ()> {
    if let Some(text) = value.as_str() {
        let mut target = declaration_target(text, runtime)?;
        target.known = known;
        targets.push(target);
        return Ok(());
    }
    let map = value.as_object().ok_or(())?;
    for (key, value) in map {
        let applies = match key.as_str() {
            "types" => Some(true),
            "default" | "import" | "require" => mode.applies(key),
            _ if key.starts_with("types@") => conditions::applicable(&key[6..], compiler),
            _ => return Err(()),
        };
        if applies == Some(false) {
            continue;
        }
        select(
            value,
            runtime && key != "types" && !key.starts_with("types@"),
            known && applies == Some(true),
            mode,
            compiler,
            targets,
        )?;
    }
    Ok(())
}

fn declaration_target(text: &str, runtime: bool) -> Result<Target, ()> {
    if [".d.ts", ".d.mts", ".d.cts"]
        .iter()
        .any(|suffix| text.ends_with(suffix))
    {
        return Ok(Target {
            path: text.to_string(),
            required: true,
            known: true,
        });
    }
    if runtime {
        for (extension, declaration) in [(".mjs", ".d.mts"), (".cjs", ".d.cts"), (".js", ".d.ts")] {
            if let Some(stem) = text.strip_suffix(extension) {
                return Ok(Target {
                    path: format!("{stem}{declaration}"),
                    required: false,
                    known: true,
                });
            }
        }
    }
    Err(())
}
