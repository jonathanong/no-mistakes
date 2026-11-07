use super::*;
use crate::codebase::dependencies::extract::{CatalogRead, ValueBinding};
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
        if same(proof, file, &import.local, producer, name)
            && (ts
                .value_flow
                .identifier_writes
                .iter()
                .any(|write| write.scope == 0 && write.name == import.local)
                || ts
                    .exported_bindings
                    .iter()
                    .any(|export| export.specifier.is_none() && export.local == import.local))
        {
            return false;
        }
        for (at, read) in usage {
            if !consumer_read(proof, file, *at, read, (producer, name)) {
                return false;
            }
        }
    }
    true
}

fn consumer_read(
    proof: &mut Proof<'_, '_>,
    file: &Path,
    at: u32,
    read: &CatalogRead,
    source: (&Path, &str),
) -> bool {
    let mut selector = read.name.clone();
    let mut remaining = read.path.as_slice();
    let mut namespaces = Vec::new();
    loop {
        if let Some((producer, name)) = proof.graph.imported_value(file, &selector) {
            return producer != source.0
                || name != source.1
                || reads::readonly(read, remaining)
                || remaining.is_empty()
                    && read.path.is_empty()
                    && reads::append_copy(proof, file, at, read);
        }
        if let Some(namespace) = proof.graph.imported_namespace(file, &selector) {
            if !namespaces.contains(&namespace) {
                namespaces.push(namespace);
            }
        }
        let Some((member, rest)) = remaining
            .split_first()
            .filter(|(member, _)| !member.is_empty() && !member.contains('.'))
        else {
            return !namespaces.iter().any(|namespace| {
                proof
                    .graph
                    .namespace_exports_value(namespace, source.0, source.1)
            });
        };
        selector.push('.');
        selector.push_str(member);
        remaining = rest;
    }
}

fn same(proof: &Proof<'_, '_>, file: &Path, selector: &str, producer: &Path, name: &str) -> bool {
    proof
        .graph
        .imported_value(file, selector)
        .is_some_and(|(path, local)| path == producer && local == name)
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
