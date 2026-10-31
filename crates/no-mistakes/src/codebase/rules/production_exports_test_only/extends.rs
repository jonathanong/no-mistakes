use crate::codebase::dependencies::extract::CallableId;
use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, NodeId};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::codebase::ts_symbols::{Export, ExportKind};
use crate::fx::{fx_set, FxHashSet};
use std::path::{Path, PathBuf};

/// Bases extended by an exported production class, keyed by declaration identity.
pub(super) struct ProductionBases {
    ids: FxHashSet<(PathBuf, CallableId)>,
}

impl ProductionBases {
    pub(super) fn collect(graph: &DepGraph, production: impl Fn(&Path) -> bool) -> Self {
        let mut ids = fx_set();
        for class in graph.class_declarations() {
            if !class.exported || class.ambient || !production(&class.file) {
                continue;
            }
            let Some(dependencies) = graph.dependencies_of_node(&class.node()) else {
                continue;
            };
            for (target, kind) in dependencies {
                if *kind != EdgeKind::Extends {
                    continue;
                }
                if let NodeId::Symbol {
                    file,
                    callable_id: Some(id),
                    ..
                } = target
                {
                    ids.insert((file.as_ref().to_path_buf(), *id));
                }
            }
        }
        Self { ids }
    }

    pub(super) fn extends_class(&self, facts: &TsFileFacts, file: &Path, export: &Export) -> bool {
        let Some(binding) = class_binding(facts, export) else {
            return false;
        };
        let Some(id) = unique_class_id(facts, binding) else {
            return false;
        };
        self.ids.contains(&(file.to_path_buf(), id))
    }
}

fn class_binding<'a>(facts: &'a TsFileFacts, export: &'a Export) -> Option<&'a str> {
    if export.is_type_only {
        return None;
    }
    let binding = match export.kind {
        ExportKind::Class | ExportKind::Default => export.name.as_str(),
        ExportKind::Const => export.local.as_deref().unwrap_or(export.name.as_str()),
        _ => return None,
    };
    facts
        .class_scopes
        .iter()
        .any(|scope| scope == binding)
        .then_some(binding)
}

/// Two classes can share a source name. Suppress only when one declaration owns it.
fn unique_class_id(facts: &TsFileFacts, scope: &str) -> Option<CallableId> {
    let mut matches = facts
        .callable_scope_ids
        .iter()
        .filter(|(_, name)| name == scope)
        .map(|(id, _)| *id);
    let id = matches.next()?;
    matches.next().is_none().then_some(id)
}

#[cfg(test)]
mod coverage {
    use super::ProductionBases;
    use crate::codebase::dependencies::graph::{DepGraph, EdgeKind, GraphBuildPlan, NodeId};
    use crate::codebase::ts_resolver::TsConfig;
    use std::path::PathBuf;

    #[test]
    fn non_extends_edges_stay_beside_a_production_base() {
        let root = crate::codebase::ts_resolver::normalize_path(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                "../../test-cases/rules/production-exports-only-used-by-tests/fixture/class-extends-production",
            ),
        );
        let graph = DepGraph::build_with_plan(
            &root,
            &TsConfig {
                dir: root.clone(),
                paths_dir: root.clone(),
                ..TsConfig::default()
            },
            GraphBuildPlan {
                extends: true,
                calls: true,
                ..GraphBuildPlan::default()
            },
        )
        .unwrap();
        let class = |scope: &str| {
            graph
                .class_declarations()
                .iter()
                .find(|class| class.scope == scope)
                .unwrap_or_else(|| panic!("{scope}"))
        };
        assert!(graph
            .dependencies_of_node(&class("GlobalChild").node())
            .is_none());
        let dependencies = graph.dependencies_of_node(&class("Sub").node()).unwrap();
        assert!(dependencies
            .iter()
            .any(|(_, kind)| *kind == EdgeKind::Extends));
        assert!(dependencies
            .iter()
            .any(|(_, kind)| *kind != EdgeKind::Extends));
        let bases = ProductionBases::collect(&graph, |path| path.starts_with(root.join("src")));
        let (target, _) = dependencies
            .iter()
            .find(|(_, kind)| *kind == EdgeKind::Extends)
            .unwrap();
        assert!(matches!(
            target,
            NodeId::Symbol {
                file,
                callable_id: Some(id),
                ..
            } if bases.ids.contains(&(file.as_ref().to_path_buf(), *id))
        ));
    }
}
