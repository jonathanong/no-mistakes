use super::selection::closed_property;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::config::v2::schema::{EnqueueFactory, EnqueueSelector, QueueFactoryRoles};
use crate::fx::FxHashMap;
use crate::queue::extract_helpers::{binding_name, export_name, static_string};
use crate::queue::extract_visitor::QueueVisitor;
use oxc_ast::ast::{
    Declaration, Expression, ImportDeclarationSpecifier, Program, Statement,
    VariableDeclarationKind,
};
use oxc_semantic::Scoping;
use oxc_syntax::symbol::SymbolId;
use std::collections::BTreeSet;

pub(in crate::queue) type Factories<'a> = FxHashMap<SymbolId, &'a EnqueueFactory>;

pub(crate) fn needs_scoping(source: &str, roles: &QueueFactoryRoles) -> bool {
    roles.enqueue.iter().any(|role| {
        role.positional().is_none() && (source.contains(&role.callee) || source.contains('\\'))
    })
}

pub(in crate::queue) fn factories<'a>(
    program: &Program<'_>,
    roles: &'a QueueFactoryRoles,
    scoping: Option<&Scoping>,
) -> Factories<'a> {
    let mut factories = Factories::default();
    if scoping.is_none() || roles.enqueue.iter().all(|role| role.positional().is_some()) {
        return factories;
    }
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind.is_type() {
            continue;
        }
        for spec in import.specifiers.iter().flatten() {
            let ImportDeclarationSpecifier::ImportSpecifier(spec) = spec else {
                continue;
            };
            if spec.import_kind.is_type() {
                continue;
            }
            let imported = export_name(&spec.imported);
            if let (Some(symbol), Some(role)) = (
                spec.local.symbol_id.get(),
                roles
                    .enqueue
                    .iter()
                    .find(|role| role.callee == imported && role.positional().is_none()),
            ) {
                factories.insert(symbol, role);
            }
        }
    }
    factories
}

pub(in crate::queue) fn collect_exports<'a>(
    visitor: &mut QueueVisitor<'a>,
    program: &'a Program<'a>,
) {
    let properties: BTreeSet<&str> = visitor
        .roles
        .enqueue
        .iter()
        .flat_map(|role| role.queue.iter().chain(std::iter::once(&role.job)))
        .filter_map(|selector| match selector {
            EnqueueSelector::Property(selector) => Some(selector.property.as_str()),
            EnqueueSelector::Positional(_) => None,
        })
        .collect();
    if properties.is_empty() {
        return;
    }
    for statement in &program.body {
        let (declaration, direct) = match statement {
            Statement::ExportDeclaration(export) => {
                let Declaration::VariableDeclaration(declaration) = &export.declaration else {
                    continue;
                };
                (declaration.as_ref(), true)
            }
            Statement::VariableDeclaration(declaration) => (declaration.as_ref(), false),
            _ => continue,
        };
        if declaration.kind != VariableDeclarationKind::Const {
            continue;
        }
        for binding in &declaration.declarations {
            let Some(name) = binding_name(&binding.id) else {
                continue;
            };
            let aliases: Vec<_> = visitor
                .facts
                .reexports
                .iter()
                .filter(|(exported, local)| {
                    local == &name && !visitor.facts.export_from.contains_key(exported)
                })
                .map(|(exported, _)| exported.as_str())
                .chain(direct.then_some(name.as_str()))
                .collect();
            if aliases.is_empty() {
                continue;
            }
            let Some(Expression::ObjectExpression(object)) =
                binding.init.as_ref().map(unwrap_ts_wrappers)
            else {
                continue;
            };
            if !visitor
                .wrapper_mutations
                .stable(binding.init.as_ref().unwrap(), &visitor.object_bindings)
            {
                continue;
            }
            let values: std::collections::HashMap<_, _> = properties
                .iter()
                .filter_map(|&key| {
                    let expr = closed_property(object, key)??;
                    Some((key.to_string(), static_string(unwrap_ts_wrappers(expr))?))
                })
                .collect();
            for alias in aliases {
                visitor
                    .facts
                    .exported_enqueue_properties
                    .insert(alias.to_string(), values.clone());
            }
        }
    }
}
