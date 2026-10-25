//! `types` and `typeRoots` from a tsconfig, following relative `extends`.
//! A later config that sets either field replaces the earlier value.
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::SourceStore;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) struct PartialTypes {
    pub(super) names: Option<(Vec<String>, PathBuf)>,
    pub(super) type_roots: Option<Vec<PathBuf>>,
}

pub(super) fn inherited(
    sources: &SourceStore,
    path: &Path,
    stack: &mut Vec<PathBuf>,
) -> Option<PartialTypes> {
    let path = normalize_path(path);
    if stack.iter().any(|seen| seen == &path) {
        return None;
    }
    let content = sources.read_path(&path).ok()?;
    let value =
        jsonc_parser::parse_to_serde_value(&content, &jsonc_parser::ParseOptions::default())
            .ok()
            .flatten()?;
    stack.push(path.clone());
    let dir = path.parent().unwrap_or(Path::new("")).to_path_buf();
    let mut acc = PartialTypes {
        names: None,
        type_roots: None,
    };
    for extend in extends_list(&value) {
        let extend = extend.replace("${configDir}", &dir.to_string_lossy());
        if !extend.starts_with('.') && !Path::new(&extend).is_absolute() {
            continue;
        }
        let parent = if Path::new(&extend).is_absolute() {
            PathBuf::from(&extend)
        } else {
            dir.join(&extend)
        };
        if let Some(parent) = inherited(sources, &parent, stack) {
            acc = merge(acc, parent);
        }
    }
    acc = merge(acc, own(&value, &dir));
    stack.pop();
    Some(acc)
}

fn merge(base: PartialTypes, overlay: PartialTypes) -> PartialTypes {
    PartialTypes {
        names: overlay.names.or(base.names),
        type_roots: overlay.type_roots.or(base.type_roots),
    }
}

fn own(value: &Value, dir: &Path) -> PartialTypes {
    let options = value.get("compilerOptions").and_then(Value::as_object);
    PartialTypes {
        names: names(options, dir),
        type_roots: type_roots(options, dir),
    }
}

fn names(
    options: Option<&serde_json::Map<String, Value>>,
    dir: &Path,
) -> Option<(Vec<String>, PathBuf)> {
    match options.and_then(|options| options.get("types")) {
        Some(Value::Array(entries)) => Some((
            entries
                .iter()
                .filter_map(|entry| entry.as_str().map(str::to_string))
                .collect(),
            dir.to_path_buf(),
        )),
        Some(_) => Some((Vec::new(), dir.to_path_buf())),
        None => None,
    }
}

fn type_roots(
    options: Option<&serde_json::Map<String, Value>>,
    dir: &Path,
) -> Option<Vec<PathBuf>> {
    match options.and_then(|options| options.get("typeRoots")) {
        Some(Value::Array(entries)) => Some(
            entries
                .iter()
                .filter_map(|entry| entry.as_str())
                .map(|entry| rooted(dir, entry))
                .collect(),
        ),
        Some(_) => Some(Vec::new()),
        None => None,
    }
}

fn rooted(dir: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    normalize_path(&if path.is_absolute() {
        path
    } else {
        dir.join(path)
    })
}

fn extends_list(value: &Value) -> Vec<String> {
    match value.get("extends") {
        Some(Value::String(value)) => vec![value.clone()],
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect(),
        _ => Vec::new(),
    }
}
