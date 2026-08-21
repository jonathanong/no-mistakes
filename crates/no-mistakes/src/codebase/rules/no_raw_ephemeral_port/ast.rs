use crate::codebase::ts_source::{static_property_key_name, unwrap_ts_wrappers};
use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, Expression, ObjectExpression, ObjectPropertyKind, Program,
};
use oxc_ast_visit::{walk, Visit};
use oxc_span::SourceType;
use std::path::Path;

pub(super) fn scan_lines(path: &Path, source: &str) -> Vec<usize> {
    let Ok(source_type) = SourceType::from_path(path) else {
        return Vec::new();
    };
    let allocator = Allocator::default();
    let parsed = crate::ast::parse(path, &allocator, source, source_type);
    collect_lines(source, &parsed.program)
}

fn collect_lines(source: &str, program: &Program<'_>) -> Vec<usize> {
    let mut visitor = ListenVisitor {
        source,
        lines: Vec::new(),
    };
    visitor.visit_program(program);
    visitor.lines.sort_unstable();
    visitor.lines.dedup();
    visitor.lines
}

struct ListenVisitor<'a> {
    source: &'a str,
    lines: Vec<usize>,
}

impl<'a> Visit<'a> for ListenVisitor<'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_ephemeral_listen(call) {
            self.lines
                .push(crate::codebase::ts_source::byte_offset_to_line(
                    self.source,
                    call.span.start as usize,
                ) as usize);
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_ephemeral_listen(call: &CallExpression<'_>) -> bool {
    is_listen_callee(&call.callee)
        && call
            .arguments
            .first()
            .is_some_and(argument_is_ephemeral_port)
}

fn is_listen_callee(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::StaticMemberExpression(member) => member.property.name.as_str() == "listen",
        Expression::ComputedMemberExpression(member) => matches!(
            unwrap_ts_wrappers(&member.expression),
            Expression::StringLiteral(literal) if literal.value.as_str() == "listen"
        ),
        _ => false,
    }
}

fn argument_is_ephemeral_port(argument: &Argument<'_>) -> bool {
    argument
        .as_expression()
        .is_some_and(|expr| expression_is_ephemeral_port(unwrap_ts_wrappers(expr)))
}

fn expression_is_ephemeral_port(expr: &Expression<'_>) -> bool {
    let expr = unwrap_ts_wrappers(expr);
    match expr {
        Expression::ObjectExpression(object) => object_has_port_zero(object),
        _ => expression_is_numeric_zero(expr),
    }
}

fn object_has_port_zero(object: &ObjectExpression<'_>) -> bool {
    object.properties.iter().any(|property| {
        let ObjectPropertyKind::ObjectProperty(property) = property else {
            return false;
        };
        static_property_key_name(&property.key) == Some("port")
            && expression_is_numeric_zero(&property.value)
    })
}

fn expression_is_numeric_zero(expr: &Expression<'_>) -> bool {
    unwrap_signed_zero(unwrap_ts_wrappers(expr)).is_number_0()
}

fn unwrap_signed_zero<'a>(expr: &'a Expression<'a>) -> &'a Expression<'a> {
    match expr {
        Expression::UnaryExpression(unary) if unary.operator.is_arithmetic() => {
            unwrap_signed_zero(unwrap_ts_wrappers(&unary.argument))
        }
        other => other,
    }
}
