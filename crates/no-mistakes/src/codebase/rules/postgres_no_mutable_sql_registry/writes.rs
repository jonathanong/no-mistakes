use super::super::postgres_sql_cross_file::{
    registry_route, unique_export, unique_namespace_member, RegistryRoutes, UniqueExport,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::ImportedBindingKind;
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::path::{Path, PathBuf};

pub(super) struct MutatedExport {
    pub producer: PathBuf,
    pub exported: String,
    pub writer: PathBuf,
}

pub(super) fn mutated_exports(
    graph: &DepGraph,
    facts: &CheckFactMap,
    routes: &RegistryRoutes<'_>,
) -> Vec<MutatedExport> {
    let mut out = Vec::new();
    for writer in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(writer) else {
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() {
            continue;
        }
        if !ts
            .imported_bindings
            .iter()
            .any(|binding| !binding.is_type_only)
            && !ts
                .value_flow
                .bindings
                .iter()
                .any(|binding| binding.owner.is_none())
        {
            continue;
        }
        let index = super::write_index::WriteIndex::new(Some(ts));
        for binding in &ts.imported_bindings {
            if binding.is_type_only {
                continue;
            }
            for member in index.namespace_writes(&binding.local) {
                push_write(
                    &mut out,
                    registry_route(graph, routes, writer, &binding.local, &[member]),
                    writer,
                );
            }
            if binding.kind == ImportedBindingKind::Namespace {
                for member in index.namespace_writes(&binding.local) {
                    push_write(
                        &mut out,
                        unique_namespace_member(graph, facts, writer, &binding.local, &member),
                        writer,
                    );
                }
                continue;
            }
            if !index.member_write(&binding.local) {
                continue;
            }
            push_write(
                &mut out,
                unique_export(graph, facts, routes, writer, &binding.local),
                writer,
            );
        }
        for binding in ts
            .value_flow
            .bindings
            .iter()
            .filter(|binding| binding.owner.is_none())
        {
            if index.member_write(&binding.name) {
                push_write(
                    &mut out,
                    registry_route(graph, routes, writer, &binding.name, &[]),
                    writer,
                );
            }
            for member in index.namespace_writes(&binding.name) {
                push_write(
                    &mut out,
                    registry_route(graph, routes, writer, &binding.name, &[member]),
                    writer,
                );
            }
        }
    }
    out.sort_by(|left, right| {
        left.producer
            .cmp(&right.producer)
            .then(left.exported.cmp(&right.exported))
            .then(left.writer.cmp(&right.writer))
    });
    out.dedup_by(|left, right| {
        left.producer == right.producer
            && left.exported == right.exported
            && left.writer == right.writer
    });
    out
}

fn push_write(out: &mut Vec<MutatedExport>, unique: Option<UniqueExport>, writer: &Path) {
    let Some(unique) = unique else {
        return;
    };
    if unique.producer == *writer {
        return;
    }
    out.push(MutatedExport {
        producer: unique.producer,
        exported: unique.exported,
        writer: writer.to_path_buf(),
    });
}
