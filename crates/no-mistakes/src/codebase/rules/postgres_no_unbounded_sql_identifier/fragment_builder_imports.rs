use super::*;
use crate::codebase::dependencies::extract::{ImportedBindingKind, ValueBinding};
use crate::codebase::dependencies::graph::{EdgeKind, NodeId};

#[path = "fragment_builder_reads.rs"]
mod reads;
pub(super) fn owned_reads(proof: &mut Proof<'_, '_>, file: &Path, binding: &ValueBinding) -> bool {
    reads::local(proof, file, binding, true)
}

pub(super) fn safe(proof: &mut Proof<'_, '_>, file: &Path, binding: &ValueBinding) -> bool {
    proof.memoized(
        Key::Imports(file.to_path_buf(), binding.name.clone()),
        |proof| {
            if !reads::local(proof, file, binding, false) {
                return false;
            }
            let Some(importers) = affected(proof, file) else {
                return false;
            };
            importers
                .iter()
                .filter(|importer| importer.as_path() != file)
                .all(|importer| consumer(proof, importer, file, &binding.name))
        },
    )
}

fn consumer(proof: &mut Proof<'_, '_>, file: &Path, producer: &Path, name: &str) -> bool {
    let Some(ts) = proof.facts.get_ts_facts(file) else {
        return false;
    };
    if proof.file(file).is_none() {
        return false;
    }
    for import in ts
        .imported_bindings
        .iter()
        .filter(|import| !import.is_type_only)
    {
        let usage: Vec<_> = ts
            .value_flow
            .catalogs
            .reads
            .iter()
            .filter(|(_, read)| read.scope == 0 && read.name == import.local)
            .collect();
        if import.kind == ImportedBindingKind::Namespace {
            for (_, read) in usage {
                let Some((member, path)) = read.path.split_first() else {
                    if !external(proof.graph, file, &import.specifier) {
                        return false;
                    }
                    continue;
                };
                if same(
                    proof,
                    file,
                    &format!("{}.{member}", import.local),
                    producer,
                    name,
                ) && !reads::readonly(read, path)
                {
                    return false;
                }
            }
        } else if same(proof, file, &import.local, producer, name) {
            if ts
                .value_flow
                .identifier_writes
                .iter()
                .any(|write| write.scope == 0 && write.name == import.local)
                || ts
                    .exported_bindings
                    .iter()
                    .any(|export| export.specifier.is_none() && export.local == import.local)
                || !usage.iter().all(|(at, read)| {
                    reads::readonly(read, &read.path)
                        || read.path.is_empty() && reads::append_copy(proof, file, **at, read)
                })
            {
                return false;
            }
        }
    }
    true
}

fn same(proof: &Proof<'_, '_>, file: &Path, selector: &str, producer: &Path, name: &str) -> bool {
    proof
        .graph
        .imported_value(file, selector)
        .is_some_and(|(path, local)| path == producer && local == name)
}

fn external(graph: &DepGraph, file: &Path, specifier: &str) -> bool {
    graph
        .dependencies_of_node(&NodeId::file(file))
        .is_some_and(|edges| {
            edges.iter().any(|(node, kind)| {
                *kind == EdgeKind::Import
                    && matches!(node, NodeId::Module(module) if &**module == specifier)
            })
        })
}

fn affected(proof: &mut Proof<'_, '_>, producer: &Path) -> Option<Vec<PathBuf>> {
    let graph = proof.graph;
    proof
        .importers
        .entry(producer.to_path_buf())
        .or_insert_with(|| {
            let mut pending = vec![producer.to_path_buf()];
            let mut seen = FxHashSet::default();
            while let Some(file) = pending.pop() {
                if !seen.insert(file.clone()) {
                    continue;
                }
                if let Some(edges) = graph.dependents_of_node(&NodeId::file(file)) {
                    for (target, kind) in edges {
                        if *kind == EdgeKind::Require {
                            return None;
                        }
                        if *kind == EdgeKind::Import {
                            if let Some(file) = target.as_file() {
                                pending.push(file.to_path_buf());
                            }
                        }
                    }
                }
            }
            let mut files: Vec<_> = seen.into_iter().collect();
            files.sort();
            Some(files)
        })
        .clone()
}
