use super::*;
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};

pub(super) fn collect<'a>(index: &index::Index<'a>, graph: &DepGraph) -> FxHashSet<Helper<'a>> {
    let mut helpers_by_file: FxHashMap<&Path, Vec<Helper<'a>>> = FxHashMap::default();
    for helper in index.accessors.keys() {
        helpers_by_file.entry(helper.0).or_default().push(*helper);
    }
    let mut escaped = FxHashSet::default();
    let mut files = FxHashSet::default();
    for name in &index.namespaces {
        if !index
            .reads
            .get(name)
            .is_some_and(|reads| reads.iter().any(|(_, read)| read.path.is_empty()))
            || !files.insert(name.0)
        {
            continue;
        }
        let mut pending = vec![name.0.to_path_buf()];
        let mut visited = FxHashSet::default();
        while let Some(file) = pending.pop() {
            if !visited.insert(file.clone()) {
                continue;
            }
            if let Some(helpers) = helpers_by_file.get(file.as_path()) {
                escaped.extend(helpers.iter().copied());
            }
            if let Some(edges) = graph.dependencies_of_node(&NodeId::file(&file)) {
                for (target, kind) in edges {
                    if matches!(kind, EdgeKind::Import | EdgeKind::Require) {
                        if let Some(file) = target.as_file() {
                            pending.push(file.to_path_buf());
                        }
                    }
                }
            }
        }
    }
    escaped
}
