//! Exact one-statement local helpers that only append to their first parameter.
use super::super::Span;
use crate::codebase::postgres::query_annotation::heads::{
    bindings::Bindings, syntax::Syntax, trusted_append,
};
use crate::codebase::ts_source::module_facts::{ModuleBinding, TypeScriptModuleFacts};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::{BindingPattern, Declaration, Expression, Function, Program, Statement};

pub(super) fn references(
    program: &Program<'_>,
    module: &TypeScriptModuleFacts,
    syntax: &Syntax,
    bindings: &Bindings<'_>,
) -> FxHashSet<Span> {
    if !syntax.dynamic_refs.is_empty() {
        return FxHashSet::default();
    }
    let observer = crate::diagnostics::current();
    if let Some(observer) = &observer {
        observer.increment("postgres.annotation.callback_append_helper_indexes", 1);
    }
    // A later declaration with the same name can replace an append-only
    // function, even if a recovered binding points at the first declaration.
    let mut functions = Vec::new();
    let mut names = FxHashMap::default();
    for statement in &program.body {
        let function = match statement {
            Statement::FunctionDeclaration(function) => function,
            Statement::ExportDeclaration(export) => {
                let Declaration::FunctionDeclaration(function) = &export.declaration else {
                    continue;
                };
                function
            }
            _ => continue,
        };
        if let Some(id) = &function.id {
            *names.entry(id.name.as_str()).or_insert(0usize) += 1;
        }
        functions.push(function);
    }
    let roots = module
        .scopes
        .iter()
        .filter(|scope| scope.parent_id.is_none())
        .map(|scope| scope.id)
        .collect::<FxHashSet<_>>();
    let mut owners: FxHashMap<Span, Option<&ModuleBinding>> = FxHashMap::default();
    let mut binding_visits = 0;
    for binding in module
        .bindings
        .iter()
        .filter(|binding| binding.runtime && !binding.imported)
    {
        binding_visits += 1;
        owners
            .entry((binding.span.start, binding.span.end))
            .and_modify(|known| *known = None)
            .or_insert(Some(binding));
    }
    if let Some(observer) = observer {
        observer.increment(
            "postgres.annotation.callback_append_helper_binding_visits",
            binding_visits,
        );
    }
    let mut allowed = FxHashSet::default();
    for function in functions {
        if function
            .id
            .as_ref()
            .is_none_or(|id| names.get(id.name.as_str()) != Some(&1))
        {
            continue;
        }
        let Some((name, parameter, receiver, tag)) = shape(function) else {
            continue;
        };
        if !bindings.declaration_safe(name)
            || !bindings.declaration_safe(parameter)
            || !bindings.declaration_safe(receiver)
        {
            continue;
        }
        if !bindings
            .owner(tag)
            .is_some_and(|owner| owner.imported && bindings.trusted_import(owner.id))
        {
            continue;
        }
        let Some(function) = owners
            .get(&name)
            .copied()
            .flatten()
            .filter(|owner| roots.contains(&owner.scope_id))
        else {
            continue;
        };
        let Some(parameter) = owners.get(&parameter).copied().flatten() else {
            continue;
        };
        if parameter.references.len() != 1
            || !parameter.references[0].runtime
            || (
                parameter.references[0].span.start,
                parameter.references[0].span.end,
            ) != receiver
            || function.references.iter().any(|usage| {
                let span = (usage.span.start, usage.span.end);
                !bindings.declaration_safe(span)
                    || syntax.assignments.writes.contains_key(&span)
                    || syntax.assignments.unsupported_writes.contains(&span)
            })
        {
            continue;
        }
        allowed.extend(
            function
                .references
                .iter()
                .filter(|usage| usage.runtime)
                .map(|usage| (usage.span.start, usage.span.end)),
        );
    }
    allowed
}

fn shape(function: &Function<'_>) -> Option<(Span, Span, Span, Span)> {
    if function.r#async
        || function.generator
        || function.params.rest.is_some()
        || function.params.items.iter().any(|param| {
            param.initializer.is_some()
                || !param.decorators.is_empty()
                || !matches!(param.pattern, BindingPattern::BindingIdentifier(_))
        })
    {
        return None;
    }
    let id = function.id.as_ref()?;
    let BindingPattern::BindingIdentifier(parameter) = &function.params.items.first()?.pattern
    else {
        return None;
    };
    let [Statement::ExpressionStatement(statement)] = function.body.as_ref()?.statements.as_slice()
    else {
        return None;
    };
    let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) else {
        return None;
    };
    if call.optional || call.arguments.len() != 1 {
        return None;
    }
    let suffix = call.arguments[0].as_expression()?;
    let Expression::TaggedTemplateExpression(tagged) = unwrap_ts_wrappers(suffix) else {
        return None;
    };
    let tag = trusted_append::tag(suffix)?;
    let allowed = function
        .params
        .items
        .iter()
        .skip(1)
        .filter_map(|param| {
            let BindingPattern::BindingIdentifier(id) = &param.pattern else {
                return None;
            };
            Some(id.name.as_str())
        })
        .collect::<FxHashSet<_>>();
    if !tagged.quasi.expressions.iter().all(|value| {
        matches!(unwrap_ts_wrappers(value), Expression::Identifier(id) if allowed.contains(id.name.as_str()))
    }) { return None }
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    if member.optional || member.property.name != "append" {
        return None;
    }
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(&member.object) else {
        return None;
    };
    (receiver.name == parameter.name).then_some((
        (id.span.start, id.span.end),
        (parameter.span.start, parameter.span.end),
        (receiver.span.start, receiver.span.end),
        tag,
    ))
}
