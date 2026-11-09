use super::{consumers::Consumed, path_provenance};
use crate::codebase::dependencies::extract::ImportKind;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId, TsFactLookup};
use crate::codebase::ts_source::module_facts::arguments::WriteScope;
use crate::fx::{FxHashMap, FxHashSet};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct Summary {
    owned: FxHashSet<String>,
    imports: FxHashSet<NodeId>,
    cycle: bool,
}

pub(super) struct ImportedWrites<'g> {
    graph: &'g DepGraph,
    summaries: RefCell<FxHashMap<PathBuf, Summary>>,
}

impl<'g> ImportedWrites<'g> {
    pub(super) fn new(graph: &'g DepGraph) -> Self {
        Self {
            graph,
            summaries: RefCell::new(FxHashMap::default()),
        }
    }

    pub(super) fn owns(
        &self,
        file: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        if consumed.unknown() || consumed.contains(file) {
            return false;
        }
        let mut summaries = self.summaries.borrow_mut();
        let summary = summaries
            .entry(file.to_path_buf())
            .or_insert_with(|| self.collect(file, facts, consumed, fixture_dirs));
        !summary.cycle && summary.owned.contains(name)
    }

    pub(super) fn module_owns_before(
        &self,
        module: &Path,
        caller: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        if consumed.unknown() || consumed.contains(module) {
            return false;
        }
        let mut summaries = self.summaries.borrow_mut();
        let summary = summaries
            .entry(module.to_path_buf())
            .or_insert_with(|| self.collect(module, facts, consumed, fixture_dirs));
        module != caller
            && !summary.cycle
            && !summary.imports.contains(&NodeId::file(caller))
            && (summary.owned.contains(name)
                || program_writes(module, facts, fixture_dirs).any(|write| write == name))
    }

    fn collect(
        &self,
        file: &Path,
        facts: &dyn TsFactLookup,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> Summary {
        let root = NodeId::file(file);
        let mut pending = vec![root.clone()];
        let mut seen = FxHashSet::default();
        seen.insert(root.clone());
        let mut summary = Summary::default();
        while let Some(node) = pending.pop() {
            let eager_workspace = node
                .as_file()
                .and_then(|file| facts.get_ts_facts(file))
                .is_some_and(|ts| {
                    ts.imports
                        .iter()
                        .all(|import| matches!(import.kind, ImportKind::Static | ImportKind::Type))
                });
            for (target, kind) in self.graph.dependencies_of_node(&node).into_iter().flatten() {
                if !(*kind == EdgeKind::Import
                    || (*kind == EdgeKind::WorkspaceImport && eager_workspace))
                {
                    continue;
                }
                if target.as_file().is_some_and(|file| consumed.contains(file)) {
                    continue;
                }
                if *target == root {
                    summary.cycle = true;
                }
                if !seen.insert(target.clone()) {
                    continue;
                }
                pending.push(target.clone());
                summary.imports.insert(target.clone());
                if let Some(file) = target.as_file() {
                    summary
                        .owned
                        .extend(program_writes(file, facts, fixture_dirs).map(str::to_owned));
                }
            }
        }
        summary
    }
}

fn program_writes<'a>(
    file: &Path,
    facts: &'a dyn TsFactLookup,
    fixture_dirs: &'a [String],
) -> impl Iterator<Item = &'a str> {
    facts
        .get_ts_facts(file)
        .and_then(path_provenance)
        .into_iter()
        .flat_map(|paths| &paths.env_writes)
        .filter(move |write| {
            write.scope == WriteScope::Program
                && super::env::owned_origin(&write.origin, fixture_dirs)
        })
        .map(|write| write.name.as_str())
}
