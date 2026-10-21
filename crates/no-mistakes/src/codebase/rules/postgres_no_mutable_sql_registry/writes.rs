use super::super::postgres_sql_cross_file::{
    registry_route, unique_export, unique_namespace_member, UniqueExport,
};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::dependencies::extract::{
    ImportedBindingKind, ValueAssignedMutations, ValueExpression, ValueMutationHistory,
};
use crate::codebase::dependencies::graph::{DepGraph, TsFactLookup};
use std::path::{Path, PathBuf};

pub(super) struct MutatedExport {
    pub producer: PathBuf,
    pub exported: String,
    pub writer: PathBuf,
}

pub(super) fn mutated_exports(graph: &DepGraph, facts: &CheckFactMap) -> Vec<MutatedExport> {
    let mut out = Vec::new();
    for writer in facts.graph_file_universe() {
        let Some(ts) = facts.get_ts_facts(writer) else {
            continue;
        };
        if ts.parse_error.is_some() || ts.operational_error.is_some() {
            continue;
        }
        for binding in &ts.imported_bindings {
            if binding.is_type_only {
                continue;
            }
            for member in namespace_writes(facts, writer, &binding.local) {
                push_write(
                    &mut out,
                    registry_route(graph, facts, writer, &binding.local, &[member]),
                    writer,
                );
            }
            if binding.kind == ImportedBindingKind::Namespace {
                for member in namespace_writes(facts, writer, &binding.local) {
                    push_write(
                        &mut out,
                        unique_namespace_member(graph, facts, writer, &binding.local, &member),
                        writer,
                    );
                }
                continue;
            }
            if !member_write(facts, writer, &binding.local) {
                continue;
            }
            push_write(
                &mut out,
                unique_export(graph, facts, writer, &binding.local),
                writer,
            );
        }
        for binding in ts
            .value_flow
            .bindings
            .iter()
            .filter(|binding| binding.owner.is_none())
        {
            if member_write(facts, writer, &binding.name) {
                push_write(
                    &mut out,
                    registry_route(graph, facts, writer, &binding.name, &[]),
                    writer,
                );
            }
            for member in namespace_writes(facts, writer, &binding.name) {
                push_write(
                    &mut out,
                    registry_route(graph, facts, writer, &binding.name, &[member]),
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

pub(super) fn member_write(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    assigned_write(facts, file, local) || property_mutator(facts, file, local)
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

fn namespace_writes(facts: &CheckFactMap, file: &Path, local: &str) -> Vec<String> {
    let Some(ts) = facts.get_ts_facts(file) else {
        return Vec::new();
    };
    let Some(histories) = &ts.value_flow.receiver_mutations else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for by_name in histories.values() {
        if let Some(history) = by_name.get(local) {
            if history.has_unassigned {
                names.extend(history.members.keys().cloned());
            }
            names.extend(
                history
                    .members
                    .iter()
                    .filter(|(_, child)| nested_write(child))
                    .map(|(name, _)| name.clone()),
            );
        }
    }
    names.sort();
    names.dedup();
    names
}

fn assigned_write(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    let Some(histories) = &ts.value_flow.receiver_mutations else {
        return false;
    };
    histories.values().any(|by_name| {
        by_name.get(local).is_some_and(|history| {
            named_members(history.assigned.as_deref())
                || history.has_unassigned
                || history.members.values().any(nested_write)
        })
    })
}

fn property_mutator(facts: &CheckFactMap, file: &Path, local: &str) -> bool {
    let Some(ts) = facts.get_ts_facts(file) else {
        return false;
    };
    ts.value_flow.calls.iter().any(|call| {
        is_property_mutator(&call.callee)
            && call
                .arguments
                .first()
                .is_some_and(|argument| is_named(argument, local))
    })
}

fn is_property_mutator(callee: &ValueExpression) -> bool {
    let ValueExpression::Member(base, method) = callee else {
        return false;
    };
    let ValueExpression::Named { name, .. } = base.as_ref() else {
        return false;
    };
    matches!(
        (name.as_str(), method.as_str()),
        (
            "Object",
            "assign" | "defineProperty" | "defineProperties" | "setPrototypeOf"
        ) | (
            "Reflect",
            "defineProperty" | "deleteProperty" | "set" | "setPrototypeOf"
        )
    )
}

fn nested_write(history: &ValueMutationHistory) -> bool {
    named_members(history.assigned.as_deref()) || history.has_unassigned
}

fn named_members(assigned: Option<&ValueAssignedMutations>) -> bool {
    assigned.is_some_and(|assigned| !assigned.members.is_empty() || assigned.dynamic.is_some())
}

fn is_named(expr: &ValueExpression, local: &str) -> bool {
    matches!(expr, ValueExpression::Named { name, .. } if name == local)
}
