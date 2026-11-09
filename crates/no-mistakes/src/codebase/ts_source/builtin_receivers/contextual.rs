use super::names::{Names, PrimitiveKind};
use super::ProvenCalls;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};

pub(super) fn collect(
    names: &Names,
    call: &CallExpression<'_>,
    receiver: &Expression<'_>,
    method: &str,
    positions: &[usize],
    proven: &mut ProvenCalls,
) {
    if !matches!(
        method,
        "map"
            | "filter"
            | "forEach"
            | "flatMap"
            | "some"
            | "every"
            | "find"
            | "findIndex"
            | "findLast"
            | "sort"
            | "toSorted"
    ) {
        return;
    }
    let Some(element) = names.array_element(receiver) else {
        return;
    };
    let calls = &mut proven.calls;
    let callbacks = &mut proven.callbacks;
    for position in positions {
        let Some(argument) = call.arguments.get(*position) else {
            continue;
        };
        match argument {
            Argument::ArrowFunctionExpression(callback) => {
                let Some(parameter) = callback.params.items.first() else {
                    continue;
                };
                let Some(name) = simple_untyped_name(parameter) else {
                    continue;
                };
                if callback_has_binding(&callback.body, name) {
                    continue;
                }
                let mut scan = ContextualCalls {
                    parameter: name,
                    element,
                    out: &mut *calls,
                    proven: &mut *callbacks,
                };
                if let Some(expression) = callback.body.as_expression() {
                    scan.visit_expression(expression);
                } else if let ArrowFunctionBody::FunctionBody(body) = &callback.body {
                    scan.visit_function_body(body);
                }
            }
            Argument::FunctionExpression(callback) => {
                let Some(parameter) = callback.params.items.first() else {
                    continue;
                };
                let Some(name) = simple_untyped_name(parameter) else {
                    continue;
                };
                let Some(body) = &callback.body else {
                    continue;
                };
                if body_has_binding(body, name) {
                    continue;
                }
                ContextualCalls {
                    parameter: name,
                    element,
                    out: &mut *calls,
                    proven: &mut *callbacks,
                }
                .visit_function_body(body);
            }
            _ => {}
        }
    }
}

fn simple_untyped_name<'n>(parameter: &'n FormalParameter<'_>) -> Option<&'n str> {
    if parameter.type_annotation.is_some() {
        return None;
    }
    match &parameter.pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

fn callback_has_binding(body: &ArrowFunctionBody<'_>, name: &str) -> bool {
    let mut scan = BindingScan { name, found: false };
    if let ArrowFunctionBody::FunctionBody(body) = body {
        scan.visit_function_body(body);
    }
    scan.found
}

fn body_has_binding(body: &FunctionBody<'_>, name: &str) -> bool {
    let mut scan = BindingScan { name, found: false };
    scan.visit_function_body(body);
    scan.found
}

struct BindingScan<'a> {
    name: &'a str,
    found: bool,
}

impl<'a> Visit<'a> for BindingScan<'_> {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.found |= identifier.name == self.name;
    }
}

struct ContextualCalls<'a, 'b> {
    parameter: &'a str,
    element: PrimitiveKind,
    out: &'b mut FxHashSet<u32>,
    proven: &'b mut FxHashMap<(u32, u32), Option<&'static [usize]>>,
}

impl<'a> Visit<'a> for ContextualCalls<'_, '_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = call.callee.without_parentheses() {
            if matches!(&member.object, Expression::Identifier(id) if id.name == self.parameter)
                && primitive_method(self.element, member.property.name.as_str())
            {
                self.out.insert(call.span.start);
                self.proven.insert((call.span.start, call.span.end), None);
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_arrow_function_expression(&mut self, _arrow: &ArrowFunctionExpression<'a>) {}
    fn visit_function(&mut self, _function: &Function<'a>, _flags: oxc_syntax::scope::ScopeFlags) {}
}

fn primitive_method(kind: PrimitiveKind, method: &str) -> bool {
    match kind {
        PrimitiveKind::Number => matches!(
            method,
            "toFixed" | "toExponential" | "toPrecision" | "toString" | "valueOf"
        ),
        PrimitiveKind::String => matches!(
            method,
            "trim"
                | "trimStart"
                | "trimEnd"
                | "padStart"
                | "padEnd"
                | "slice"
                | "substring"
                | "toLowerCase"
                | "toUpperCase"
                | "toString"
                | "valueOf"
        ),
        PrimitiveKind::Boolean | PrimitiveKind::BigInt => matches!(method, "toString" | "valueOf"),
    }
}
