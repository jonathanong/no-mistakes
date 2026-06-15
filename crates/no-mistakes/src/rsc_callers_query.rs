//! `rsc-callers <component-file>`: find the server components and pages that
//! transitively import a component through React Server Component boundaries.
//!
//! Traversal walks the reverse import graph from the component. A `"use client"`
//! file is a client boundary: it is not reported (the query wants *server*
//! callers) and the upward RSC chain stops there, because everything rendered
//! above a client boundary renders the client subtree, not the target directly.
//! Files with `"use server"` or no directive (App Router defaults to server
//! components) are reported and traversal continues through them.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::codebase::dependencies::graph::{DepGraph, GraphBuildPlan};
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::{find_tsconfig, load_tsconfig, normalize_path, TsConfig};
use crate::codebase::ts_source::relative_slash_path;

/// React Server Component environment of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Server,
    Client,
    Unknown,
}

/// File "kind" in App Router terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CallerKind {
    Page,
    Component,
}

/// One server component/page that transitively imports the target.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct RscCaller {
    pub file: String,
    pub kind: CallerKind,
    pub environment: Environment,
    pub depth: usize,
}

/// The full `rsc-callers` report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RscCallersReport {
    pub component: String,
    pub callers: Vec<RscCaller>,
}

impl RscCallersReport {
    /// Sorted unique caller file paths, for `--format paths`.
    pub fn paths(&self) -> Vec<String> {
        let mut paths: Vec<String> = self.callers.iter().map(|c| c.file.clone()).collect();
        paths.sort();
        paths.dedup();
        paths
    }
}

/// Next.js App Router special files that are routable "pages".
const PAGE_STEMS: &[&str] = &[
    "page",
    "route",
    "layout",
    "template",
    "default",
    "loading",
    "error",
    "not-found",
];

fn resolve_tsconfig(root: &Path, tsconfig: Option<&Path>) -> Result<TsConfig> {
    match tsconfig {
        Some(path) => load_tsconfig(path),
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

fn runtime_edge(kind: EdgeKind) -> bool {
    matches!(
        kind,
        EdgeKind::Import | EdgeKind::DynamicImport | EdgeKind::Require
    )
}

/// Run the `rsc-callers` query.
pub fn run(
    root: &Path,
    config_path: Option<&Path>,
    tsconfig: Option<&Path>,
    component: &Path,
    depth: Option<usize>,
) -> Result<RscCallersReport> {
    let root = normalize_path(root);
    let component_abs = if component.is_absolute() {
        component.to_path_buf()
    } else {
        root.join(component)
    };
    let component_node = NodeId::File(normalize_path(&component_abs));

    let tsconfig = resolve_tsconfig(&root, tsconfig)?;
    let graph =
        DepGraph::build_with_plan_and_config(&root, &tsconfig, GraphBuildPlan::all(), config_path)?;

    let mut env_cache: HashMap<PathBuf, Environment> = HashMap::new();
    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut callers: Vec<RscCaller> = Vec::new();
    let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();

    visited.insert(component_node.clone());
    queue.push_back((component_node, 0));

    while let Some((node, node_depth)) = queue.pop_front() {
        let Some(importers) = graph.dependents_of_node(&node) else {
            continue;
        };
        for (importer, kind) in importers {
            if !runtime_edge(*kind) {
                continue;
            }
            // Reverse import edges of a file always resolve to file importers.
            let NodeId::File(path) = importer else {
                continue;
            };
            if !visited.insert(importer.clone()) {
                continue;
            }
            let environment = *env_cache
                .entry(path.clone())
                .or_insert_with(|| detect_environment(path));
            let importer_depth = node_depth + 1;
            if environment == Environment::Client {
                // Client boundary: not a server caller, and the RSC chain stops.
                continue;
            }
            callers.push(RscCaller {
                file: relative_slash_path(&root, path),
                kind: caller_kind(path),
                environment,
                depth: importer_depth,
            });
            if depth.is_none_or(|max| importer_depth < max) {
                queue.push_back((importer.clone(), importer_depth));
            }
        }
    }

    callers.sort();
    callers.dedup();

    Ok(RscCallersReport {
        component: relative_slash_path(&root, &component_abs),
        callers,
    })
}

fn detect_environment(path: &Path) -> Environment {
    let Ok(source) = std::fs::read_to_string(path) else {
        return Environment::Unknown;
    };
    crate::ast::with_program(path, &source, |program, _| {
        let has_use_server = program
            .directives
            .iter()
            .any(|directive| directive.directive == "use server");
        let has_use_client = program
            .directives
            .iter()
            .any(|directive| directive.directive == "use client");
        match (has_use_server, has_use_client) {
            (true, _) => Environment::Server,
            (_, true) => Environment::Client,
            _ => Environment::Unknown,
        }
    })
    .unwrap_or(Environment::Unknown)
}

fn caller_kind(path: &Path) -> CallerKind {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("");
    if PAGE_STEMS.contains(&stem) {
        CallerKind::Page
    } else {
        CallerKind::Component
    }
}

#[cfg(test)]
mod tests;
