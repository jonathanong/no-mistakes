//! Anonymous `export default` string constants.
//!
//! The dependency extractor records a bare `export default QUERY` under
//! `QUERY`, and every other expression default under `default`. A namespace
//! member (`ns.CONST`) is not a hole this walk can fill.

use super::super::super::super::tags::quasi_text;
use super::super::super::ScopeVisitor;
use crate::codebase::postgres::embedded::helpers::{sentinel, ExportedConstant};
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use oxc_ast::ast::{ExportDefaultDeclaration, ExportDefaultDeclarationKind, Expression};

/// Local name `record_local_export_binding` uses for an expression default.
const DEFAULT_LOCAL: &str = "default";

pub(super) fn record(visitor: &mut ScopeVisitor<'_>, export: &ExportDefaultDeclaration<'_>) {
    if let ExportDefaultDeclarationKind::Identifier(ident) = &export.declaration {
        super::record_export(visitor, Some(ident.name.as_str()));
        return;
    }
    let Some(expression) = export.declaration.as_expression() else {
        return;
    };
    // Parentheses and type assertions are not bare identifiers, so locate
    // still calls the binding `default`.
    let expression = expression.get_inner_expression();
    let constant = if let Expression::Identifier(ident) = expression {
        super::exported_constant(visitor, ident.name.as_str())
    } else {
        literal_constant(visitor, expression)
    };
    let Some(constant) = constant else {
        return;
    };
    visitor
        .exported_constants
        .insert(DEFAULT_LOCAL.to_string(), constant);
}

fn literal_constant(
    visitor: &ScopeVisitor<'_>,
    expression: &Expression<'_>,
) -> Option<ExportedConstant> {
    Some(ExportedConstant {
        // Template interpolation rejects fragment constants. A string default
        // has to stay a string so `${query}` can splice it.
        fragment: false,
        sql: RecoveredSql::literal(expression, visitor.source, literal_sql(expression)?),
    })
}

fn literal_sql(expression: &Expression<'_>) -> Option<String> {
    let sql = match expression {
        Expression::StringLiteral(literal) => crate::ast::js_literal(literal).into_owned(),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => template
            .quasis
            .iter()
            .map(|quasi| quasi_text(quasi, false))
            .collect(),
        _ => return None,
    };
    (!sql.contains('\0') && !sentinel::contains(&sql)).then_some(sql)
}
