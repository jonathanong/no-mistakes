//! One resolved production-import projection for a rule application.

use super::{resolve_target, ReachabilityContext};
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::EdgeKind;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::edge_index::{CanonicalEdge, EdgeIndex};
use crate::fx::{FxHashMap, FxHashSet};
use rayon::prelude::*;
use std::path::PathBuf;

pub(in crate::codebase::rules::production_dependency_declarations) struct PreparedReachability {
    pub(super) edges: EdgeIndex<PathBuf, EdgeKind>,
    pub(super) external_seeds: FxHashMap<PathBuf, FxHashSet<PathBuf>>,
}

impl PreparedReachability {
    pub(in crate::codebase::rules::production_dependency_declarations) fn new(
        ctx: &ReachabilityContext<'_>,
        sources: &SourceStore,
    ) -> Self {
        // Ordinary DepGraph imports use tsconfig catalogs, extension rewrites,
        // and workspace membership. This rule's manually scoped workspace and
        // literal extension priority differ, so those edges cannot substitute.
        // Resolve each distinct (source, specifier) once, retaining typed edges
        // in the canonical EdgeIndex shape instead of a second adjacency map.
        let mut resolved = ctx
            .imports_by_file
            .par_iter()
            .map(|(file, imports)| {
                let mut targets = FxHashMap::default();
                let mut edges = Vec::new();
                for import in imports {
                    let kind = match import.kind {
                        ImportKind::Static => EdgeKind::Import,
                        ImportKind::Dynamic => EdgeKind::DynamicImport,
                        ImportKind::Require => EdgeKind::Require,
                        ImportKind::RequireResolve => EdgeKind::RequireResolve,
                        ImportKind::Type => continue,
                    };
                    let target = targets.entry(import.specifier.as_str()).or_insert_with(|| {
                        if let Some(observer) = sources.observer() {
                            observer.increment("production_dependencies.resolutions", 1);
                        }
                        resolve_target(ctx.workspace, file, &import.specifier, ctx.visible)
                    });
                    if let Some(target) = target {
                        edges.push(CanonicalEdge::new(file.clone(), target.clone(), kind));
                    }
                }
                (file, edges)
            })
            .collect::<Vec<_>>();
        resolved.sort_unstable_by(|left, right| left.0.cmp(right.0));
        let edges = EdgeIndex::from_edges(resolved.into_iter().flat_map(|(_, edges)| edges));
        let mut external_seeds: FxHashMap<PathBuf, FxHashSet<PathBuf>> = FxHashMap::default();
        // Group external production seeds by target owner in one pass. Package
        // BFS below never scans another package's imports or resolves an edge.
        for edge in edges.edges() {
            let Some(owner) = ctx.owners.get(&edge.to) else {
                continue;
            };
            if ctx.owners.get(&edge.from) == Some(owner)
                || ctx
                    .test_globset
                    .is_match(relative_slash_path(ctx.root, &edge.from))
            {
                continue;
            }
            external_seeds
                .entry(owner.clone())
                .or_default()
                .insert(edge.to.clone());
        }
        Self {
            edges,
            external_seeds,
        }
    }
}
