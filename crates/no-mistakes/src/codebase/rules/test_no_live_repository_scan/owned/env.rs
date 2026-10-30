//! Which `process.env` variables are only ever set to an owned temp root.
use super::path_provenance;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::module_facts::arguments::{PathAnchor, PathProvenance, WriteScope};
use crate::config::v2::schema::StringOrList;
use crate::config::v2::NoMistakesConfig;
use crate::fx::FxHashMap;
use std::path::{Path, PathBuf};

#[derive(Default)]
pub(super) struct EnvWrites {
    /// A write this pass cannot attribute to a variable, or a file it cannot read.
    poisoned: bool,
    /// Per variable: whether every write anywhere is an owned temp root.
    owned: FxHashMap<String, bool>,
}

impl EnvWrites {
    pub(super) fn new(
        facts: &dyn TsFactLookup,
        graph_files: &[PathBuf],
        root: &Path,
        config: &NoMistakesConfig,
    ) -> Self {
        let mut writes = Self::default();
        // A file with no path facts was outside the rule's scope; a file whose
        // facts failed to build may hide writes.
        for file in graph_files {
            let Some(ts) = facts.get_ts_facts(file) else {
                continue;
            };
            if ts
                .module_bindings
                .as_deref()
                .is_none_or(|module| module.path_provenance.is_none())
            {
                continue;
            }
            let provenance = path_provenance(ts);
            writes.poisoned |= provenance.is_none_or(|found| found.env_unknown_write);
            writes.poisoned |=
                runner_config(file) && provenance.is_some_and(|found| found.opaque_config_key);
            for write in provenance.iter().flat_map(|found| &found.env_writes) {
                let entry = writes.owned.entry(write.name.clone()).or_insert(true);
                *entry &= write.origin.anchor == PathAnchor::Temp
                    && !write.origin.segments.iter().any(|segment| segment == "..");
            }
        }
        writes.poisoned |= json_configs_poison(root, config);
        writes
    }

    pub(super) fn all_owned(&self, name: &str) -> bool {
        !self.poisoned && self.owned.get(name).copied().unwrap_or(false)
    }
}

/// An unconditional owned write of `name` that must have run before `offset`.
pub(in super::super) fn dominated(provenance: &PathProvenance, name: &str, offset: u32) -> bool {
    provenance
        .env_writes
        .iter()
        .filter(|write| {
            write.name == name
                && write.origin.anchor == PathAnchor::Temp
                && !write.origin.segments.iter().any(|segment| segment == "..")
        })
        .any(|write| match write.scope {
            WriteScope::Program => write.end <= offset,
            WriteScope::Function(start, end) => {
                start <= offset && offset < end && write.end <= offset
            }
            // A hook runs before every test body registered in the same scope.
            WriteScope::Hook(start, end) => provenance
                .test_callbacks
                .iter()
                .any(|(from, to)| start <= *from && *to <= end && *from <= offset && offset < *to),
            WriteScope::Conditional => false,
        })
}

fn runner_config(path: &Path) -> bool {
    matches!(
        path.file_stem().and_then(|stem| stem.to_str()),
        Some("vitest.config" | "vitest.workspace" | "vitest.projects" | "vite.config")
    )
}

fn json_configs_poison(root: &Path, config: &NoMistakesConfig) -> bool {
    let mut paths = config
        .tests
        .vitest
        .configs
        .as_ref()
        .map(StringOrList::values)
        .unwrap_or_default()
        .into_iter()
        .map(|value| {
            let path = PathBuf::from(value);
            if path.is_absolute() {
                path
            } else {
                root.join(path)
            }
        })
        .collect::<Vec<_>>();
    for name in [
        "vitest.workspace.json",
        "vitest.workspace.jsonc",
        "vitest.projects.json",
        "vitest.projects.jsonc",
    ] {
        paths.push(root.join(name));
    }
    paths.into_iter().any(|path| json_file_poisons(&path))
}

fn json_file_poisons(path: &Path) -> bool {
    if !matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("json" | "jsonc")
    ) {
        return false;
    }
    match std::fs::read_to_string(path) {
        Ok(source) => json_text_poisons(&source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(_) => true,
    }
}

pub(in super::super) fn json_text_poisons(source: &str) -> bool {
    match jsonc_parser::parse_to_serde_value(source, &jsonc_parser::ParseOptions::default()) {
        Ok(Some(value)) => contains_unstub(&value),
        Ok(None) => false,
        Err(_) => true,
    }
}

fn contains_unstub(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            map.keys()
                .any(|key| matches!(key.as_str(), "unstubEnvs" | "unstubAllEnvs"))
                || map.values().any(contains_unstub)
        }
        serde_json::Value::Array(items) => items.iter().any(contains_unstub),
        _ => false,
    }
}
