use crate::queue::extract_helpers::{binding_name, export_name};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    Argument, CallExpression, Declaration, ExportDeclaration, ExportNamedDeclaration, Expression,
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
    {
        visitor
            .enqueue_factories
            .insert(local.to_string(), (spec.queue, spec.job));
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

pub(super) fn note_queue_binding(
    visitor: &mut QueueVisitor<'_>,
    name: &str,
    call: &CallExpression<'_>,
) {
    let Expression::Identifier(id) = &call.callee else {
        return;
    };
    let Some(&index) = visitor.queue_name_index.get(id.name.as_str()) else {
        return;
    };
    let Some(arg) = call.arguments.get(index) else {
        return;
    };
    if let Some(queue_name) = visitor.literal_arg(arg) {
        visitor
            .facts
            .queue_bindings
            .insert(name.to_string(), queue_name.clone());
        visitor
            .facts
            .queue_exports
            .insert(name.to_string(), queue_name);
        return;
    }
    if let Some(imported) = identifier_arg(arg) {
        if visitor.facts.string_imports.contains_key(&imported) {
            visitor
                .facts
                .pending_queue_bindings
                .insert(name.to_string(), imported);
        }
    }
}

pub(super) fn note_alias(visitor: &mut QueueVisitor<'_>, name: &str, local: &str) {
    if let Some(queue_name) = visitor.facts.queue_bindings.get(local).cloned() {
        visitor
            .facts
            .queue_bindings
            .insert(name.to_string(), queue_name.clone());
        visitor
            .facts
            .queue_exports
            .insert(name.to_string(), queue_name);
        return;
    }
    if visitor.facts.string_imports.contains_key(local) {
        visitor
            .facts
            .reexports
            .push((name.to_string(), local.to_string()));
    }
}

pub(super) fn note_export(visitor: &mut QueueVisitor<'_>, decl: &ExportNamedDeclaration<'_>) {
    for spec in &decl.specifiers {
        let local = export_name(&spec.local);
        let exported = export_name(&spec.exported);
        if visitor.facts.queue_bindings.contains_key(&local) {
            if let Some(queue_name) = visitor.facts.queue_bindings.get(&local).cloned() {
                visitor.facts.queue_exports.insert(exported, queue_name);
            }
            continue;
        }
        visitor.facts.reexports.push((exported, local));
    }
}

pub(super) fn note_exported_declaration(
    visitor: &mut QueueVisitor<'_>,
    decl: &ExportDeclaration<'_>,
) {
    let Declaration::VariableDeclaration(var_decl) = &decl.declaration else {
        return;
    };
    for declarator in &var_decl.declarations {
        let (Some(name), Some(Expression::StringLiteral(value))) =
            (binding_name(&declarator.id), declarator.init.as_ref())
        else {
            continue;
        };
        visitor
            .facts
            .exported_strings
            .insert(name, value.value.as_str().to_string());
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

pub(super) fn identifier_arg(arg: &Argument<'_>) -> Option<String> {
    match arg {
        Argument::Identifier(id) => Some(id.name.as_str().to_string()),
        _ => match arg.as_expression() {
            Some(Expression::Identifier(id)) => Some(id.name.as_str().to_string()),
            _ => None,
        },
    }
}

#[path = "extract_factory_calls.rs"]
mod calls;
#[path = "extract_factory_dedup.rs"]
mod dedup;
#[path = "extract_factory_walk.rs"]
mod job_scope;

pub(super) use calls::{note_factory_call, note_wait};
pub(super) use dedup::note_self_dedup;
pub(super) use job_scope::{visit_arrow, visit_function, visit_switch};
