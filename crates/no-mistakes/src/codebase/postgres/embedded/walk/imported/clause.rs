//! A same-file literal-string helper used inside an ordinary SQL template.
//! Imported calls inside its program are resolved only by the prepared pass.

use super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{sentinel, ImportedSite, SiteKind};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use oxc_ast::ast::{Argument, CallExpression, Expression};

pub(in crate::codebase::postgres::embedded::walk) fn piece(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
) -> Option<String> {
    let Expression::Identifier(name) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let name = name.name.as_str();
    if visitor.head_calls_only
        || call.optional
        || visitor.shadowed_locally(name)
        || visitor.imported_names.contains(name)
        || !visitor.functions.has_program(name)
        || visitor.functions.has_static_helper(name)
        || visitor.functions.is_parameter_builder(name)
        || !call.arguments.iter().all(plain_argument)
    {
        return None;
    }
    let args = super::clause_args(visitor, call)?;
    if visitor.functions.identifier_clauses(name, &args).is_some() {
        return None;
    }
    let line = byte_offset_to_line(visitor.source, call.span.start as usize);
    let index = visitor.imported_sites.borrow_mut().intern(
        call.span.start,
        ImportedSite {
            local: name.to_string(),
            member: None,
            kind: SiteKind::PlainClause,
            args,
            value_used: true,
            line,
            constant: None,
        },
    );
    Some(sentinel::encode(index))
}

// An argument expression must not call a getter or mutate SQL before the read.
fn plain_argument(argument: &Argument<'_>) -> bool {
    let Some(expr) = argument.as_expression() else {
        return false;
    };
    match unwrap_ts_wrappers(expr) {
        Expression::Identifier(name) => name.name != "undefined",
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_) => true,
        Expression::TemplateLiteral(template) => template.expressions.is_empty(),
        _ => false,
    }
}
