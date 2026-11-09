use super::names::{Names, PrimitiveKind};
use super::ProvenCalls;
use crate::fx::{FxHashMap, FxHashSet};
use oxc_ast::ast::*;
use oxc_ast_visit::{walk, Visit};
mod callback_validation;

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
            | "findLastIndex"
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
                for parameter in callback_parameters(method, &callback.params.items) {
                    let Some(name) = simple_untyped_name(parameter) else {
                        continue;
                    };
                    if callback_validation::arrow_has_binding(&callback.body, name)
                        || callback_validation::arrow_has_write(&callback.body, name)
                    {
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
            }
            Argument::FunctionExpression(callback) => {
                if let Some(body) = &callback.body {
                    for parameter in callback_parameters(method, &callback.params.items) {
                        let Some(name) = simple_untyped_name(parameter) else {
                            continue;
                        };
                        if callback_validation::body_has_binding(body, name)
                            || callback_validation::body_has_write(body, name)
                        {
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
                }
            }
            _ => {}
        }
    }
}

fn callback_parameters<'p, 'ast>(
    method: &str,
    parameters: &'p [FormalParameter<'ast>],
) -> impl Iterator<Item = &'p FormalParameter<'ast>> {
    parameters
        .iter()
        .take(if matches!(method, "sort" | "toSorted") {
            2
        } else {
            1
        })
}

fn simple_untyped_name<'n, 'ast>(parameter: &'n FormalParameter<'ast>) -> Option<&'n str> {
    if parameter.type_annotation.is_some() {
        return None;
    }
    match &parameter.pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
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
            if self
                .contextual_kind(&member.object, 0)
                .is_some_and(|kind| primitive_method(kind, member.property.name.as_str()))
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

impl ContextualCalls<'_, '_> {
    fn contextual_kind(&self, expression: &Expression<'_>, depth: u8) -> Option<PrimitiveKind> {
        if depth > 8 {
            return None;
        }
        match expression.without_parentheses() {
            Expression::Identifier(identifier) if identifier.name == self.parameter => {
                Some(self.element)
            }
            Expression::CallExpression(call) => {
                let Expression::StaticMemberExpression(member) = call.callee.without_parentheses()
                else {
                    return None;
                };
                let receiver = self.contextual_kind(&member.object, depth + 1)?;
                (receiver == PrimitiveKind::String
                    && callback_validation::string_result_method(member.property.name.as_str()))
                .then_some(PrimitiveKind::String)
            }
            _ => None,
        }
    }
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
                | "localeCompare"
        ),
        PrimitiveKind::Boolean | PrimitiveKind::BigInt => matches!(method, "toString" | "valueOf"),
    }
}
