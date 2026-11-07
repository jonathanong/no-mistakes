use super::*;
use oxc_ast::ast::{BindingPattern, CallExpression, Expression};
use oxc_span::GetSpan;
use oxc_syntax::operator::BinaryOperator;

pub(super) fn note_call(collector: &mut ImportCollector, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return;
    };
    let Some(name) = names::identifier_name(&member.object) else {
        return;
    };
    if collector.metadata_registry_names.contains(&name)
        && !collector.metadata_lookup_calls.contains(&call.span.start)
    {
        collector.record_registry_mutation(&name);
    }
}

pub(super) fn note_guard(
    collector: &mut ImportCollector,
    statement: &oxc_ast::ast::Statement<'_>,
    until: u32,
) {
    let oxc_ast::ast::Statement::IfStatement(branch) = statement else {
        return;
    };
    if branch.alternate.is_some()
        || super::super::completion::statement_completes(&branch.consequent)
    {
        return;
    }
    let Expression::UnaryExpression(unary) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&branch.test)
    else {
        return;
    };
    if unary.operator != oxc_syntax::operator::UnaryOperator::LogicalNot {
        return;
    }
    let Some(name) = names::identifier_name(&unary.argument) else {
        return;
    };
    let mut candidates = collector.value_flow.bindings.iter().filter(|binding| {
        binding.name == name
            && binding.owner == collector.current_function_id()
            && binding.declared_at < statement.span().start
            && collector.value_flow.bounded_bindings.iter().any(|bounded| {
                bounded.requires_guard
                    && bounded.name == name
                    && bounded.scope == binding.scope
                    && bounded.declared_at == binding.declared_at
            })
    });
    let Some(binding) = candidates.next() else {
        return;
    };
    if candidates.next().is_some() {
        return;
    }
    collector
        .value_flow
        .finite_identifier_guards
        .push(FiniteIdentifierGuard {
            context: collector.current_function_id(),
            name,
            members: Vec::new(),
            from: statement.span().end,
            until,
            kind: GuardKind::Lookup,
            subject: Some(GuardSubject::Binding {
                scope: binding.scope,
                declared_at: binding.declared_at,
            }),
        });
}

pub(super) fn note_lookup(
    collector: &mut ImportCollector,
    name: &str,
    init: &Expression<'_>,
    scope: usize,
    declared_at: u32,
) {
    let Some(call) = plain_call(init) else {
        return;
    };
    let Some(collection) = lookup_collection(call) else {
        return;
    };
    let Some(registry) = collector
        .value_flow
        .closed_string_registries
        .iter()
        .find(|registry| {
            registry.name == collection && registry.kind == RegistryKind::MetadataArray
        })
    else {
        return;
    };
    if !unique_closed_collection(
        &collector.value_flow,
        &collection,
        RegistryKind::MetadataArray,
    ) {
        return;
    }
    let source = Some((collection, registry.scope));
    let fields = registry.keys.clone();
    collector.metadata_lookup_calls.insert(call.span.start);
    rows::note_lookup(collector, name, scope);
    for key in fields {
        collector.value_flow.bounded_bindings.push(BoundedBinding {
            name: name.to_string(),
            scope,
            declared_at,
            source: source.clone(),
            members: vec![key],
            requires_guard: true,
        });
    }
}

pub(super) fn lookup_collection(call: &CallExpression<'_>) -> Option<String> {
    let Expression::StaticMemberExpression(member) =
        crate::codebase::ts_source::unwrap_ts_wrappers(&call.callee)
    else {
        return None;
    };
    if call.optional
        || member.optional
        || member.property.name != "find"
        || call.arguments.len() != 1
    {
        return None;
    }
    let Expression::ArrowFunctionExpression(arrow) =
        crate::codebase::ts_source::unwrap_ts_wrappers(call.arguments[0].as_expression()?)
    else {
        return None;
    };
    if arrow.r#async || arrow.params.items.len() != 1 || arrow.params.rest.is_some() {
        return None;
    }
    let BindingPattern::BindingIdentifier(parameter) = &arrow.params.items[0].pattern else {
        return None;
    };
    let expression = arrow.body.as_expression()?;
    let Expression::BinaryExpression(binary) =
        crate::codebase::ts_source::unwrap_ts_wrappers(expression)
    else {
        return None;
    };
    if !matches!(
        binary.operator,
        BinaryOperator::StrictEquality | BinaryOperator::StrictInequality
    ) {
        return None;
    }
    let parameter = parameter.name.as_str();
    if !(member_of(&binary.left, parameter) && scalar(&binary.right)
        || member_of(&binary.right, parameter) && scalar(&binary.left))
    {
        return None;
    }
    names::identifier_name(&member.object)
}

fn member_of(expr: &Expression<'_>, parameter: &str) -> bool {
    matches!(crate::codebase::ts_source::unwrap_ts_wrappers(expr), Expression::StaticMemberExpression(member) if !member.optional && names::identifier_name(&member.object).as_deref() == Some(parameter))
}

fn scalar(expr: &Expression<'_>) -> bool {
    matches!(
        crate::codebase::ts_source::unwrap_ts_wrappers(expr),
        Expression::Identifier(_) | Expression::StringLiteral(_)
    )
}
