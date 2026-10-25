//! Ambient modules named by a tsconfig `types` list. Unset `types` does not
//! scan installed packages. Relative names resolve beside the config that set them.
use super::ambient::AmbientModules;
use super::package;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_resolver::{normalize_path, TsConfigCatalog};
use crate::codebase::ts_source::facts::{TsFactPlan, TsFileFacts};
use crate::codebase::ts_source::SourceStore;
use std::path::{Path, PathBuf};
#[path = "type_config.rs"]
mod type_config;

pub(super) fn visible_modules(
    session: &AnalysisSession,
    root: &Path,
    sources: &SourceStore,
    catalog: Option<&TsConfigCatalog>,
    prepared: Option<&dyn TsFactLookup>,
) -> AmbientModules {
    let Some(catalog) = catalog else {
        return AmbientModules::default();
    };
    let mut modules = AmbientModules::default();
    let mut seen = crate::fx::fx_set();
    for (index, config) in catalog.config_paths() {
        let mut stack = Vec::new();
        let Some(selection) = type_config::inherited(sources, &config, &mut stack) else {
            continue;
        };
        let Some((names, names_dir)) = selection.names else {
            continue;
        };
        for name in names {
            let Some(file) = resolve_name(
                &name,
                &names_dir,
                selection.type_roots.as_deref(),
                &config,
                root,
                sources,
            ) else {
                continue;
            };
            if !seen.insert((index, file.clone())) {
                continue;
            }
            if let Some(facts) = reusable_facts(prepared, &file) {
                modules.index_file(Some(index), &file, facts);
                continue;
            }
            let facts = session.declaration_facts(root, &file);
            modules.index_file(Some(index), &file, &facts);
        }
    }
    modules
}

/// Prepared facts already walked this file. A second collection would parse it again.
fn reusable_facts<'a>(
    prepared: Option<&'a dyn TsFactLookup>,
    file: &Path,
) -> Option<&'a TsFileFacts> {
    let prepared = prepared?;
    if !prepared.covers_ts_fact_plan(TsFactPlan {
        function_calls: true,
        ..TsFactPlan::default()
    }) {
        return None;
    }
    prepared.get_ts_facts(file)
}

fn resolve_name(
    name: &str,
    names_dir: &Path,
    type_roots: Option<&[PathBuf]>,
    importer: &Path,
    consumer_root: &Path,
    sources: &SourceStore,
) -> Option<PathBuf> {
    if name.starts_with("./") || name.starts_with("../") || Path::new(name).is_absolute() {
        let path = if Path::new(name).is_absolute() {
            PathBuf::from(name)
        } else {
            names_dir.join(name)
        };
        let path = normalize_path(&path);
        return sources.read_path(&path).ok().map(|_| path);
    }
    let roots;
    let roots = match type_roots {
        Some(roots) => roots,
        None => {
            roots = default_type_roots(names_dir);
            &roots
        }
    };
    for type_root in roots {
        if let Some(path) = package_entry(&type_root.join(name), sources) {
            return Some(path);
        }
    }
    package::resolve_in_context(name, importer, consumer_root, sources)
}

fn default_type_roots(dir: &Path) -> Vec<PathBuf> {
    dir.ancestors()
        .filter_map(|ancestor| {
            let candidate = ancestor.join("node_modules/@types");
            candidate.is_dir().then_some(candidate)
        })
        .collect()
}

fn package_entry(dir: &Path, sources: &SourceStore) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }
    if let Ok(manifest) = sources.parse_json_path(&dir.join("package.json")) {
        if let Some(relative) = manifest
            .get("types")
            .or_else(|| manifest.get("typings"))
            .and_then(|value| value.as_str())
        {
            let path = normalize_path(&dir.join(relative));
            return sources.read_path(&path).ok().map(|_| path);
        }
    }
    for name in ["index.d.ts", "index.d.mts", "index.d.cts"] {
        let path = normalize_path(&dir.join(name));
        if sources.read_path(&path).is_ok() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
#[path = "type_entries_tests.rs"]
mod type_entries_tests;
