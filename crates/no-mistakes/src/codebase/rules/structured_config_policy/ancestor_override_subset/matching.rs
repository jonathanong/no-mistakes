use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde_yaml::{Mapping, Value};
use std::path::{Component, Path};

pub(super) fn compile_value_globs(value: &Mapping, key: &str) -> Option<GlobSet> {
    compile_globs(mapping_value(value, key)?)
}

pub(super) fn optional_value_globs(value: &Mapping, key: &str) -> Result<Option<GlobSet>, ()> {
    let Some(value) = mapping_value(value, key) else {
        return Ok(None);
    };
    compile_globs(value).ok_or(()).map(Some)
}

pub(super) fn mapping_value<'a>(value: &'a Mapping, key: &str) -> Option<&'a Value> {
    value.get(Value::String(key.to_string()))
}

pub(super) fn relative_path(base_dir: &Path, path: &Path) -> Option<String> {
    let base = base_dir.components().collect::<Vec<_>>();
    let path = path.components().collect::<Vec<_>>();
    if base.first() != path.first() {
        return None;
    }
    let common = base
        .iter()
        .zip(&path)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = Vec::new();
    relative.extend(base[common..].iter().filter_map(|component| {
        matches!(component, Component::Normal(_)).then_some("..".to_string())
    }));
    relative.extend(
        path[common..]
            .iter()
            .filter_map(|component| match component {
                Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
                _ => None,
            }),
    );
    Some(relative.join("/"))
}

fn compile_globs(value: &Value) -> Option<GlobSet> {
    let patterns = match value {
        Value::String(pattern) => vec![pattern.as_str()],
        Value::Sequence(patterns) => patterns
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()?,
        _ => return None,
    };
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let trimmed = pattern.trim_start_matches("./");
        builder.add(
            GlobBuilder::new(trimmed)
                .literal_separator(true)
                .build()
                .ok()?,
        );
    }
    builder.build().ok()
}
