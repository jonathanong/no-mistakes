use crate::codebase::dependencies::graph::{ClassDeclaration, DepGraph, EdgeKind, NodeId};
use std::path::Path;

pub(super) fn extended_by_production(
    graph: &DepGraph,
    file: &Path,
    name: &str,
    production: impl Fn(&Path) -> bool,
) -> bool {
    graph.class_declarations().iter().any(|class| {
        class.exported
            && !class.ambient
            && production(&class.file)
            && extends_export(graph, class, file, name)
    })
}

fn extends_export(graph: &DepGraph, class: &ClassDeclaration, file: &Path, name: &str) -> bool {
    graph
        .dependencies_of_node(&class.node())
        .into_iter()
        .flatten()
        .any(|(target, kind)| {
            *kind == EdgeKind::Extends
                && matches!(
                    target,
                    NodeId::Symbol { file: base_file, symbol, .. }
                        if base_file.as_ref() == file && symbol.as_ref() == name
                )
        })
}
