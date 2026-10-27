use super::*;
use oxc_ast::ast::{
    BindingPattern, Expression, ObjectExpression, ObjectPropertyKind, VariableDeclarator,
};
use oxc_span::GetSpan;

pub(super) fn variable(runtime: &mut Runtime<'_>, variable: &VariableDeclarator<'_>) {
    if let (BindingPattern::BindingIdentifier(binding), Some(init)) =
        (&variable.id, variable.init.as_ref())
    {
        if let Some(id) = runtime.bindings.get(&span(binding.span)).copied() {
            let fields = match unwrap_ts_wrappers(init) {
                Expression::ObjectExpression(object) => Some(field_names(object, runtime.possible)),
                Expression::Identifier(source) => runtime
                    .bindings
                    .get(&span(source.span))
                    .and_then(|source| runtime.object_fields.get(source).cloned()),
                _ => None,
            };
            if let Some(fields) = fields.filter(|fields| !fields.is_empty()) {
                runtime.object_fields.insert(id, fields);
            }
        }
    }
    walk::walk_variable_declarator(runtime, variable);
}

pub(super) fn visit(runtime: &mut Runtime<'_>, object: &ObjectExpression<'_>) {
    let outer_fields = runtime.this_fields.clone();
    let fields = field_names(object, runtime.possible);
    for property in &object.properties {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            if let ObjectPropertyKind::SpreadProperty(spread) = property {
                runtime.this_fields = outer_fields.clone();
                runtime.visit_expression(&spread.argument);
            }
            continue;
        };
        runtime.this_fields = outer_fields.clone();
        runtime.visit_property_key(&property.key);
        if let Expression::FunctionExpression(function) = unwrap_ts_wrappers(&property.value) {
            let enabled = runtime.enabled;
            runtime.enabled = None;
            runtime.this_fields = Some(fields.clone());
            walk::walk_function(runtime, function, oxc_syntax::scope::ScopeFlags::Function);
            runtime.enabled = enabled;
            runtime.this_fields = outer_fields.clone();
        } else {
            runtime.this_fields = outer_fields.clone();
            runtime.visit_expression(&property.value);
        }
    }
    runtime.this_fields = outer_fields;
}

pub(super) fn call(runtime: &mut Runtime<'_>, call: &oxc_ast::ast::CallExpression<'_>) {
    let start = runtime.reads.len();
    walk::walk_call_expression(runtime, call);
    for read in runtime.reads[start..].iter().copied() {
        if executor_callee(&call.callee, read)
            || call.arguments.iter().any(|argument| {
                argument
                    .as_expression()
                    .is_some_and(|value| contains(span(value.span()), read))
            })
        {
            runtime.uses.insert(read);
        }
    }
    runtime.reads.truncate(start);
}

fn field_names(object: &ObjectExpression<'_>, possible: &FxHashSet<Span>) -> FxHashSet<String> {
    object
        .properties
        .iter()
        .filter_map(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return None;
            };
            (possible_value(&property.value, possible))
                .then(|| crate::ast::property_key_name(&property.key).map(|name| name.to_string()))
                .flatten()
        })
        .collect()
}

fn possible_value(expression: &Expression<'_>, possible: &FxHashSet<Span>) -> bool {
    match expression {
        Expression::TSAsExpression(value) => {
            super::super::possible::contains(possible, &value.type_annotation)
                || possible_value(&value.expression, possible)
        }
        Expression::TSSatisfiesExpression(value) => {
            super::super::possible::contains(possible, &value.type_annotation)
                || possible_value(&value.expression, possible)
        }
        Expression::TSTypeAssertion(value) => {
            super::super::possible::contains(possible, &value.type_annotation)
                || possible_value(&value.expression, possible)
        }
        Expression::ParenthesizedExpression(value) => possible_value(&value.expression, possible),
        _ => false,
    }
}

fn executor_callee(callee: &Expression<'_>, read: Span) -> bool {
    match unwrap_ts_wrappers(callee) {
        Expression::StaticMemberExpression(member)
            if executor_member(member.property.name.as_str()) =>
        {
            contains(span(member.object.span()), read)
        }
        Expression::ComputedMemberExpression(member) if matches!(unwrap_ts_wrappers(&member.expression), Expression::StringLiteral(key) if executor_member(crate::ast::js_literal(key).as_ref())) => {
            contains(span(member.object.span()), read)
        }
        _ => span(callee.span()) == read,
    }
}

fn contains(outer: Span, inner: Span) -> bool {
    outer.0 <= inner.0 && inner.1 <= outer.1
}

fn executor_member(name: &str) -> bool {
    matches!(name, "query" | "call" | "apply" | "bind")
}
