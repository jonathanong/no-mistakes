use crate::queue::extract_helpers::{binding_name, export_name, static_string};
use crate::queue::extract_model::LocalQueue;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    Argument, CallExpression, Declaration, ExportAllDeclaration, ExportDeclaration,
    ExportFromDeclaration, ExportNamedDeclaration, Expression,
};
pub(super) fn note_named_import(
    visitor: &mut QueueVisitor<'_>,
    imported: &str,
    local: &str,
    source: &str,
) {
    visitor.facts.string_imports.insert(
        local.to_string(),
        (source.to_string(), imported.to_string()),
    );
    if let Some(index) = queue_name_index(visitor, imported) {
        visitor.queue_name_index.insert(local.to_string(), index);
    }
    if let Some(spec) = visitor
        .roles
        .enqueue
        .iter()
        .find(|spec| spec.callee == imported)
        .and_then(|spec| spec.positional())
    {
        visitor.enqueue_factories.insert(local.to_string(), spec);
    }
    if let Some(spec) = visitor
        .roles
        .worker
        .iter()
        .find(|spec| spec.callee == imported)
    {
        visitor
            .worker_factories
            .insert(local.to_string(), (spec.name, spec.processor));
    }
}
pub(super) fn note_queue_binding<'a>(
    visitor: &mut QueueVisitor<'a>,
    name: &str,
    call: &CallExpression<'a>,
    declaration: u32,
) -> bool {
    let Expression::Identifier(id) = &call.callee else {
        return false;
    };
    let Some(&index) = visitor.queue_name_index.get(id.name.as_str()) else {
        return false;
    };
    let Some(arg) = call.arguments.get(index) else {
        return false;
    };
    let (queue_name, queue_import) = visitor.resolve_queue_name_arg(arg);
    if let Some(queue_name) = queue_name.clone() {
        visitor.note_queue_declaration(name, declaration, LocalQueue::Known(queue_name));
    } else if let Some(imported) = queue_import {
        visitor.note_queue_declaration(name, declaration, LocalQueue::ImportedName(imported));
    } else {
        return false;
    }
    super::job_options::note_factory_defaults(
        visitor,
        declaration,
        queue_name.as_deref(),
        index,
        call,
    );
    true
}
pub(super) fn note_export(visitor: &mut QueueVisitor<'_>, decl: &ExportNamedDeclaration<'_>) {
    if decl.export_kind.is_type() {
        return;
    }
    for spec in &decl.specifiers {
        if spec.export_kind.is_type() {
            continue;
        }
        let local = export_name(&spec.local);
        let exported = export_name(&spec.exported);
        if visitor.facts.queue_bindings.contains_key(&local) {
            if let Some(queue_name) = visitor.facts.queue_bindings.get(&local).cloned() {
                visitor
                    .facts
                    .queue_exports
                    .insert(exported.clone(), queue_name);
            }
            visitor.facts.reexports.push((exported, local));
            continue;
        }
        visitor.facts.reexports.push((exported, local));
    }
}
pub(super) fn note_export_from(visitor: &mut QueueVisitor<'_>, decl: &ExportFromDeclaration<'_>) {
    if decl.export_kind.is_type() {
        return;
    }
    let specifier = crate::ast::js_literal(&decl.source);
    for spec in &decl.specifiers {
        if spec.export_kind.is_type() {
            continue;
        }
        let imported = export_name(&spec.local);
        let exported = export_name(&spec.exported);
        // The exported name is not a local binding. Lookup follows it through `specifier`.
        visitor
            .facts
            .export_from
            .insert(exported.clone(), (specifier.to_string(), imported));
        visitor.facts.reexports.push((exported.clone(), exported));
    }
}
pub(super) fn note_export_all(visitor: &mut QueueVisitor<'_>, decl: &ExportAllDeclaration<'_>) {
    // `export type *` and `export * as ns` do not re-export value names.
    if decl.export_kind.is_type() || decl.exported.is_some() {
        return;
    }
    visitor
        .facts
        .star_exports
        .push(crate::ast::js_literal(&decl.source).into_owned());
}
pub(super) fn note_exported_declaration(
    visitor: &mut QueueVisitor<'_>,
    decl: &ExportDeclaration<'_>,
) {
    let Declaration::VariableDeclaration(var_decl) = &decl.declaration else {
        return;
    };
    for declarator in &var_decl.declarations {
        let (Some(name), Some(init)) = (binding_name(&declarator.id), declarator.init.as_ref())
        else {
            continue;
        };
        let Some(value) = static_string(init) else {
            continue;
        };
        visitor.facts.exported_strings.insert(name, value);
    }
}

fn queue_name_index(visitor: &QueueVisitor<'_>, imported: &str) -> Option<usize> {
    if let Some(spec) = visitor
        .roles
        .queue
        .iter()
        .find(|spec| spec.callee == imported)
    {
        return Some(spec.name);
    }
    visitor
        .configured_factory_names
        .iter()
        .any(|name| name == imported)
        .then_some(0)
}
pub(in crate::queue) fn identifier_arg(arg: &Argument<'_>) -> Option<String> {
    match arg.as_expression() {
        Some(Expression::Identifier(id)) => Some(id.name.as_str().to_string()),
        _ => None,
    }
}

#[path = "extract_factory_calls.rs"]
mod calls;
#[path = "extract_factory_dedup.rs"]
mod dedup;
#[path = "extract_factory_index.rs"]
mod index;
#[path = "extract_factory_walk.rs"]
mod job_scope;
#[path = "extract_factory_scope.rs"]
mod scope;

pub(super) use calls::{note_factory_call, note_wait, scheduler_factories};
pub(super) use dedup::{dedup_id_token, note_self_dedup, visit_variable_declaration};
pub(super) use index::queue_index;
pub(super) use job_scope::{visit_arrow, visit_function, visit_switch};
pub(super) use scope::{visit_block, visit_body, visit_catch, visit_class};
pub(super) use scope::{visit_for, visit_for_in, visit_for_of, visit_static_block};
