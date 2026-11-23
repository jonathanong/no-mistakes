use super::super::selection::closed_property;
use crate::codebase::ts_source::unwrap_ts_wrappers as unwrap;
use crate::config::v2::schema::EnqueueSelector;
use crate::queue::extract_helpers::static_string;
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{Declaration, Expression, Program, Statement, VariableDeclarationKind};
use std::collections::{BTreeSet, HashMap};

pub(in crate::queue) fn collect_exports<'a>(
    visitor: &mut QueueVisitor<'a>,
    program: &'a Program<'a>,
) {
    if visitor
        .roles
        .enqueue
        .iter()
        .all(|role| role.positional().is_some())
    {
        return;
    }
    let properties: BTreeSet<&str> = visitor
        .roles
        .enqueue
        .iter()
        .flat_map(|role| role.queue.iter().chain(std::iter::once(&role.job)))
        .filter_map(|selector| match selector {
            EnqueueSelector::Property(selector) => Some(selector.property.as_str()),
            _ => None,
        })
        .collect();
    let mut aliases = HashMap::<String, Vec<String>>::new();
    for (exported, local) in &visitor.facts.reexports {
        if !visitor.facts.export_from.contains_key(exported) {
            visitor
                .facts
                .explicit_enqueue_exports
                .insert(exported.clone());
            aliases
                .entry(local.clone())
                .or_default()
                .push(exported.clone());
        }
    }
    visitor
        .facts
        .explicit_enqueue_exports
        .extend(visitor.facts.export_from.keys().cloned());
    for (local, exported) in &aliases {
        if visitor.facts.string_imports.contains_key(local) {
            if visitor.wrapper_mutations.stable_import(local) {
                for name in exported {
                    visitor
                        .facts
                        .enqueue_export_aliases
                        .insert(name.clone(), local.clone());
                }
            } else {
                visitor
                    .facts
                    .blocked_enqueue_exports
                    .extend(exported.iter().cloned());
            }
        }
    }
    for statement in &program.body {
        let (declaration, direct) = match statement {
            Statement::ExportDeclaration(export) => {
                if export.declaration.is_type() {
                    continue;
                }
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    if let Some(id) = export.declaration.id() {
                        visitor
                            .facts
                            .explicit_enqueue_exports
                            .insert(id.name.to_string());
                    }
                    continue;
                };
                (declaration.as_ref(), true)
            }
            Statement::VariableDeclaration(declaration) => (declaration.as_ref(), false),
            Statement::ExportDefaultDeclaration(_) => {
                visitor
                    .facts
                    .explicit_enqueue_exports
                    .insert("default".to_string());
                continue;
            }
            _ => continue,
        };
        for binding in &declaration.declarations {
            for id in binding.id.get_binding_identifiers() {
                let mut exported = aliases.get(id.name.as_str()).cloned().unwrap_or_default();
                if direct {
                    exported.push(id.name.to_string());
                }
                visitor
                    .facts
                    .explicit_enqueue_exports
                    .extend(exported.iter().cloned());
                if exported.is_empty() {
                    continue;
                }
                let init = binding.init.as_ref().map(unwrap);
                if declaration.kind != VariableDeclarationKind::Const
                    || !visitor.wrapper_mutations.stable_local(
                        id.name.as_str(),
                        binding.span.end,
                        &visitor.object_bindings,
                    )
                    || init.is_none_or(|expr| {
                        !visitor
                            .wrapper_mutations
                            .stable(expr, &visitor.object_bindings)
                    })
                {
                    visitor.facts.blocked_enqueue_exports.extend(exported);
                    continue;
                }
                if !matches!(
                    &binding.id,
                    oxc_ast::ast::BindingPattern::BindingIdentifier(_)
                ) {
                    continue;
                }
                let init = init.unwrap();
                let string = static_string(init);
                let values = match init {
                    Expression::ObjectExpression(object) => properties
                        .iter()
                        .filter_map(|&key| {
                            Some((
                                key.to_string(),
                                static_string(unwrap(closed_property(object, key)??))?,
                            ))
                        })
                        .collect(),
                    _ => HashMap::new(),
                };
                for name in exported {
                    if let Some(value) = &string {
                        visitor
                            .facts
                            .stable_exported_strings
                            .insert(name.clone(), value.clone());
                    }
                    visitor
                        .facts
                        .exported_enqueue_properties
                        .insert(name, values.clone());
                }
            }
        }
    }
}
