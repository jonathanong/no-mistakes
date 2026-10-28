//! Exact inline callback receiver census; unrelated callback effects cannot use the SQL value.
use super::Span;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use crate::fx::FxHashSet;
use oxc_ast::ast::{self, BindingPattern, Expression, ObjectPropertyKind, PropertyKind};
use oxc_ast_visit::{walk, Visit};
use oxc_syntax::scope::ScopeFlags;

pub(super) fn executor(
    value: &Expression<'_>,
    executors: &FxHashSet<Span>,
    append_helpers: &FxHashSet<Span>,
) -> Option<(Span, bool)> {
    let Expression::ArrowFunctionExpression(arrow) = unwrap_ts_wrappers(value) else {
        return None;
    };
    if arrow.params.rest.is_some() || arrow.params.items.len() != 1 {
        return None;
    }
    let param = &arrow.params.items[0];
    if param.initializer.is_some() || !param.decorators.is_empty() {
        return None;
    }
    let BindingPattern::BindingIdentifier(param) = &param.pattern else {
        return None;
    };
    if matches!(param.name.as_str(), "arguments" | "eval") {
        return None;
    }
    let mut census = Census {
        name: param.name.as_str(),
        executors,
        append_helpers,
        allowed: FxHashSet::default(),
        references: Vec::new(),
        discarded: FxHashSet::default(),
        endpoint: None,
        count: 0,
        valid: true,
    };
    census.visit_arrow_function_body(&arrow.body);
    (census.valid
        && census.count == 1
        && census
            .references
            .iter()
            .all(|span| census.allowed.contains(span)))
    .then_some(census.endpoint?)
}

struct Census<'a> {
    name: &'a str,
    executors: &'a FxHashSet<Span>,
    append_helpers: &'a FxHashSet<Span>,
    allowed: FxHashSet<u32>,
    references: Vec<u32>,
    discarded: FxHashSet<u32>,
    endpoint: Option<(Span, bool)>,
    count: usize,
    valid: bool,
}

impl<'a> Visit<'a> for Census<'_> {
    fn visit_call_expression(&mut self, call: &ast::CallExpression<'a>) {
        if call.optional
            || call
                .arguments
                .iter()
                .any(|arg| arg.as_expression().is_none())
        {
            self.valid = false;
        }
        let first = call
            .arguments
            .first()
            .and_then(|arg| arg.as_expression())
            .map(unwrap_ts_wrappers);
        if let Some(Expression::Identifier(arg)) =
            first.filter(|arg| matches!(arg, Expression::Identifier(id) if id.name == self.name))
        {
            let span = (call.span.start, call.span.end);
            if self.executors.contains(&span) && call.arguments.len() <= 2 {
                let needs_fresh = call.arguments.len() == 2;
                if !needs_fresh || options(call.arguments[1].as_expression().unwrap()) {
                    self.count += 1;
                    self.endpoint = Some((span, needs_fresh));
                    self.allowed.insert(arg.span.start);
                } else {
                    self.valid = false
                }
            } else if let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) {
                if self
                    .append_helpers
                    .contains(&(callee.span.start, callee.span.end))
                {
                    self.allowed.insert(arg.span.start);
                }
            }
        }
        if let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) {
            if !member.optional
                && member.property.name == "append"
                && call.arguments.len() == 1
                && self.discarded.contains(&call.span.start)
            {
                if let Expression::Identifier(receiver) = unwrap_ts_wrappers(&member.object) {
                    if receiver.name == self.name {
                        self.allowed.insert(receiver.span.start);
                    }
                }
            }
        }
        walk::walk_call_expression(self, call);
    }
    fn visit_expression_statement(&mut self, statement: &ast::ExpressionStatement<'a>) {
        if let Expression::CallExpression(call) = unwrap_ts_wrappers(&statement.expression) {
            self.discarded.insert(call.span.start);
        }
        walk::walk_expression_statement(self, statement);
    }
    fn visit_identifier_reference(&mut self, id: &ast::IdentifierReference<'a>) {
        if matches!(id.name.as_str(), "eval" | "arguments") {
            self.valid = false
        }
        if id.name == self.name {
            self.references.push(id.span.start)
        }
    }
    fn visit_binding_identifier(&mut self, id: &ast::BindingIdentifier<'a>) {
        if id.name == self.name {
            self.valid = false
        }
    }
    fn visit_assignment_expression(&mut self, _: &ast::AssignmentExpression<'a>) {
        self.valid = false
    }
    fn visit_update_expression(&mut self, _: &ast::UpdateExpression<'a>) {
        self.valid = false
    }
    fn visit_function(&mut self, _: &ast::Function<'a>, _: ScopeFlags) {
        self.valid = false
    }
    fn visit_arrow_function_expression(&mut self, _: &ast::ArrowFunctionExpression<'a>) {
        self.valid = false
    }
    fn visit_class(&mut self, _: &ast::Class<'a>) {
        self.valid = false
    }
}

fn options(value: &Expression<'_>) -> bool {
    let Expression::ObjectExpression(object) = unwrap_ts_wrappers(value) else {
        return false;
    };
    object.properties.iter().all(|entry| match entry {
        ObjectPropertyKind::SpreadProperty(spread) => {
            matches!(
                unwrap_ts_wrappers(&spread.argument),
                Expression::Identifier(_)
            )
        }
        ObjectPropertyKind::ObjectProperty(property) => {
            !property.computed
                && !property.method
                && property.kind == PropertyKind::Init
                && matches!(
                    unwrap_ts_wrappers(&property.value),
                    Expression::BooleanLiteral(_)
                )
        }
    })
}
