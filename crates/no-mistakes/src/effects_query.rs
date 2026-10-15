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
use serde::Serialize;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;

/// One distinct configured match for a physical effect occurrence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct EffectCallAlternative {
    pub callee: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One matched physical effect call site.
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
    /// Sorted distinct matches; omitted for ordinary singleton occurrences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<EffectCallAlternative>>,
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

pub(crate) struct EffectsSelection {
    kind: String,
    names: HashMap<String, Option<String>>,
    provenance: crate::effect_provenance::EffectProvenance,
    returned: crate::fx::FxHashMap<
        String,
        crate::fx::FxHashMap<String, crate::config::v2::schema::EffectTargetConfig>,
    >,
    targets: crate::fx::FxHashMap<
        String,
        crate::fx::FxHashMap<String, crate::config::v2::schema::EffectTargetConfig>,
    >,
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
    let root = root.canonicalize().unwrap_or(root);
    let entry_abs = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        root.join(entry)
    };
    let allowed = runtime_edges();
    let crate::codebase::dependencies::query::PreparedQueryGraph {
        selection,
        graph,
        facts,
        interner,
    } = crate::codebase::dependencies::query::prepare_query_graph(
        crate::codebase::dependencies::query::QueryGraphRequest {
            root: &root,
            config_path,
            tsconfig_path: tsconfig,
            explicit_root: &entry_abs,
            config_before_facts: true,
            fact_plan: crate::codebase::ts_source::facts::TsFactPlan {
                imports: true,
                function_calls: true,
                effect_calls: true,
                ..Default::default()
            },
            select: |config: Option<&crate::config::v2::schema::NoMistakesConfig>| {
                selection_from_config(
                    config.expect("effects config is loaded before selection"),
                    kind,
                    categories,
                )
            },
            validate_root:
                |_: &EffectsSelection, _: &mut crate::codebase::dependencies::graph::GraphFiles| {
                    if !entry_abs.is_file() {
                        bail!("entry file not found: {}", entry_abs.display());
                    }
                    Ok(())
                },
            graph_plan: |selection: &EffectsSelection| GraphBuildPlan {
                calls: needs_resolved_effects(selection),
                effect_selectors: !selection.targets.is_empty(),
                ..GraphBuildPlan::from_allowed(Some(&allowed))
            },
            configure_context:
                |selection: &EffectsSelection,
                 context: &mut crate::codebase::ts_source::facts::TsFactContext| {
                    context.effect_functions = selection.names.clone()
                },
            selection: std::marker::PhantomData,
        },
    )?;

    run_with_prepared(&root, &selection, entry, depth, &graph, &facts, &interner)
}

include!("effects_query/selection.rs");
include!("effects_query/matches.rs");
include!("effects_query/prepared.rs");

#[cfg(test)]
mod tests;
