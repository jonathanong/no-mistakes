use super::*;
use crate::codebase::ts_source::facts::{
    collect_file_facts_from_program, TsFactContext, TsFactPlan,
};
use crate::codebase::ts_source::{discover_visible_paths, FileInventory, SourceStore};
use crate::codebase::workspaces::load_indexed_from_source_store;
use crate::diagnostics::{with_observer, InvocationObserver};
use std::cell::RefCell;
use std::collections::BTreeMap;

struct Fixture {
    root: PathBuf,
    workspace: IndexedWorkspaceMap,
    visible: PathSet,
    files: FileIdMap<CheckFileFacts>,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = crate::codebase::ts_resolver::normalize_path(
            &crate::test_support::rule_fixture_root("postgres-require-query-annotation").join(name),
        );
        let paths = discover_visible_paths(&root);
        let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&paths)));
        let workspace = load_indexed_from_source_store(&root, &sources).unwrap();
        let files = paths
            .iter()
            .filter(|path| path.extension().is_some_and(|ext| ext == "mts"))
            .map(|path| {
                let source = sources.read_path(path).unwrap();
                let facts = crate::ast::with_program(path, &source, |program, _| {
                    collect_file_facts_from_program(
                        path,
                        TsFactPlan {
                            imports: true,
                            module_bindings: true,
                            ..Default::default()
                        },
                        &TsFactContext::default(),
                        &source,
                        program,
                        None,
                        None,
                    )
                })
                .unwrap();
                (
                    path.clone(),
                    CheckFileFacts {
                        ts: Arc::new(facts),
                        ..Default::default()
                    },
                )
            })
            .collect();
        Self {
            root,
            workspace,
            visible: paths.into_iter().collect(),
            files,
        }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn fallback<'a>(&'a self, seeds: Option<&'a PathSet>) -> WorkspaceFallback<'a> {
        WorkspaceFallback::new(&self.workspace, &self.visible, seeds, &self.files)
    }
}

fn assert_cached(fallback: &WorkspaceFallback<'_>) {
    for (from, targets) in &fallback.prepared {
        for (specifier, target) in targets {
            assert_eq!(
                fallback.resolve(specifier, from, |_, _| panic!("cached edge resolved again")),
                target.path().cloned()
            );
        }
    }
}

mod graph;
mod observers;
mod permissions;
