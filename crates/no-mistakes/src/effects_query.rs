//! `effects <kind> --entry <file>`: report every transitive call site of a
//! configured set of effect functions/constructors that is reachable from
//! `<entry>` through the import graph.
//!
//! The function/constructor names per `<kind>` come entirely from configuration
//! (`effects.<kind>` in `.no-mistakes.yml`); nothing is hardcoded. Reachability
//! reuses the canonical dependency graph ([`DepGraph::deps_of`]) over runtime
//! import edges, then each reachable file is parsed once to collect matching
//! call sites with line numbers.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use rayon::prelude::*;
use serde::Serialize;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::{find_tsconfig, load_tsconfig, normalize_path, TsConfig};
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::load_v2_config;

mod extract;

/// One matched effect call site.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EffectCallSite {
    pub file: String,
    pub line: usize,
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<String>,
    pub depth: usize,
}

/// The full `effects <kind>` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectsReport {
    pub kind: String,
    pub entry: String,
    pub call_sites: Vec<EffectCallSite>,
    pub by_category: BTreeMap<String, usize>,
}

impl EffectsReport {
    /// Sorted unique matched file paths, for `--format paths`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self
            .call_sites
            .iter()
            .map(|site| site.file.clone())
            .collect();
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Edge kinds that represent runtime reachability (code that actually executes
/// when the entry module is imported). Type-only imports are excluded.
fn runtime_edges() -> HashSet<EdgeKind> {
    HashSet::from([
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        // Workspace-package imports are runtime imports in a monorepo.
        EdgeKind::WorkspaceImport,
    ])
}

fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    match tsconfig {
        // Resolve a relative explicit tsconfig against `root`, not the cwd.
        Some(path) if path.is_absolute() => load_tsconfig(path),
        Some(path) => load_tsconfig(&root.join(path)),
        None => match find_tsconfig(root) {
            Some(path) => load_tsconfig(&path),
            None => Ok(TsConfig {
                dir: root.to_path_buf(),
                paths: vec![],
                paths_dir: root.to_path_buf(),
                base_url: None,
            }),
        },
    }
}

/// Run the `effects <kind>` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    kind: &str,
    entry: &Path,
    categories: &[String],
    depth: Option<usize>,
) -> Result<EffectsReport> {
    let root = normalize_path(root);
    let config = load_v2_config(&root, config_path)?;
    let Some(kind_config) = config.effects.get(kind) else {
        let available: Vec<&str> = config.effects.keys().map(String::as_str).collect();
        bail!(
            "unknown effects kind: {kind} (configured kinds: {})",
            if available.is_empty() {
                "<none>".to_string()
            } else {
                available.join(", ")
            }
        );
    };

    // name -> category label (None for the flat `functions` list).
    let mut names: HashMap<String, Option<String>> = HashMap::new();
    for (category, functions) in &kind_config.categories {
        if !categories.is_empty() && !categories.iter().any(|c| c == category) {
            continue;
        }
        for function in functions {
            names.insert(function.clone(), Some(category.clone()));
        }
    }
    if categories.is_empty() {
        for function in &kind_config.functions {
            names.entry(function.clone()).or_insert(None);
        }
    }
    if names.is_empty() {
        bail!("effects kind `{kind}` has no functions for the requested categories");
    }

    let entry_abs = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        root.join(entry)
    };
    if !entry_abs.is_file() {
        bail!("entry file not found: {}", entry_abs.display());
    }
    let entry_node = NodeId::File(normalize_path(&entry_abs));

    let tsconfig = resolve_tsconfig(&root, tsconfig)?;
    let graph =
        DepGraph::build_with_plan_and_config(&root, &tsconfig, GraphBuildPlan::all(), config_path)?;
    let allowed = runtime_edges();
    let reachable = graph.deps_of(std::slice::from_ref(&entry_node), depth, Some(&allowed));

    // Map every reachable file (plus the entry itself at depth 0) to its depth.
    let mut file_depths: HashMap<PathBuf, usize> = HashMap::new();
    if let NodeId::File(path) = &entry_node {
        file_depths.insert(path.clone(), 0);
    }
    // `deps_of` yields each node once at its minimum depth, so a first-wins
    // insert preserves the shallowest depth without a redundant merge.
    for entry in &reachable {
        if let NodeId::File(path) = &entry.node {
            file_depths.entry(path.clone()).or_insert(entry.depth);
        }
    }

    let mut call_sites: Vec<EffectCallSite> = file_depths
        .par_iter()
        .flat_map(|(path, depth)| extract::scan_file(&root, path, *depth, &names))
        .collect();
    call_sites.sort();

    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for site in &call_sites {
        let label = site
            .category
            .clone()
            .unwrap_or_else(|| "uncategorized".to_string());
        *by_category.entry(label).or_insert(0) += 1;
    }

    Ok(EffectsReport {
        kind: kind.to_string(),
        entry: relative_slash_path(&root, &entry_abs),
        call_sites,
        by_category,
    })
}

#[cfg(test)]
mod tests;
