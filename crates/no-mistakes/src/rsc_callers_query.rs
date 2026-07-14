//! `rsc-callers <component-file>`: find the server components and pages that
//! transitively import a component through React Server Component boundaries.
//!
//! Traversal walks the reverse import graph from the component. A `"use client"`
//! file is a client boundary: it is not reported (the query wants *server*
//! callers) and the upward RSC chain stops there, because everything rendered
//! above a client boundary renders the client subtree, not the target directly.
//! Files with `"use server"` or no directive (App Router defaults to server
//! components) are reported and traversal continues through them.

use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Serialize;

use crate::codebase::dependencies::graph::DepGraph;
use crate::codebase::dependencies::{EdgeKind, NodeId};
use crate::codebase::ts_resolver::normalize_path;
use crate::codebase::ts_source::relative_slash_path;

/// React Server Component environment of a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Server,
    Client,
    Unknown,
}

impl From<crate::codebase::ts_source::facts::RscEnvironmentFact> for Environment {
    fn from(value: crate::codebase::ts_source::facts::RscEnvironmentFact) -> Self {
        match value {
            crate::codebase::ts_source::facts::RscEnvironmentFact::Server => Self::Server,
            crate::codebase::ts_source::facts::RscEnvironmentFact::Client => Self::Client,
            crate::codebase::ts_source::facts::RscEnvironmentFact::Unknown => Self::Unknown,
        }
    }
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

mod prepare;
pub use prepare::run;

fn runtime_edge(kind: EdgeKind) -> bool {
    matches!(
        kind,
        EdgeKind::Import
            | EdgeKind::DynamicImport
            | EdgeKind::Require
            // Workspace-package imports are runtime imports in a monorepo.
            | EdgeKind::WorkspaceImport
    )
}

fn runtime_edges() -> HashSet<EdgeKind> {
    HashSet::from([
        EdgeKind::Import,
        EdgeKind::DynamicImport,
        EdgeKind::Require,
        EdgeKind::WorkspaceImport,
    ])
}

pub(crate) fn run_with_prepared(
    root: &Path,
    component: &Path,
    depth: Option<usize>,
    graph: &DepGraph,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
) -> Result<RscCallersReport> {
    let component_abs = if component.is_absolute() {
        component.to_path_buf()
    } else {
        root.join(component)
    };
    if !component_abs.is_file() {
        anyhow::bail!("component file not found: {}", component_abs.display());
    }
    let component_node = NodeId::File(normalize_path(&component_abs));

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut callers: Vec<RscCaller> = Vec::new();
    let mut queue: VecDeque<(NodeId, usize)> = VecDeque::new();

    visited.insert(component_node.clone());
    queue.push_back((component_node, 0));

    while let Some((node, node_depth)) = queue.pop_front() {
        let Some(importers) = graph.dependents_of_node(&node) else {
            continue;
        };
        // Reverse runtime-import edges of a file always resolve to file importers.
        let file_importers = importers
            .iter()
            .filter(|(_, kind)| runtime_edge(*kind))
            .filter_map(|(importer, _)| importer.as_file().map(|path| (importer, path)));
        for (importer, path) in file_importers {
            if !visited.insert(importer.clone()) {
                continue;
            }
            let environment = facts
                .get(path)
                .and_then(|file| file.rsc_environment)
                .map(Environment::from)
                .unwrap_or(Environment::Unknown);
            let importer_depth = node_depth + 1;
            // `--depth` is a hard result limit: a caller beyond it is not reported.
            if depth.is_some_and(|max| importer_depth > max) {
                continue;
            }
            if environment == Environment::Client {
                // Client boundary: not a server caller, and the RSC chain stops.
                continue;
            }
            callers.push(RscCaller {
                file: relative_slash_path(root, path),
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
        component: relative_slash_path(root, &component_abs),
        callers,
    })
}

#[cfg(test)]
mod test_support;

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
