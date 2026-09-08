mod append;
mod chain;
mod functions;

pub(super) use append::apply_append;
pub(super) use functions::LocalFunctions;

use super::super::tags::{interpolating_untrusted_tag, kind_for_const};
use super::super::{first_call_argument, sql_text, EmbeddedSqlCall, EmbeddedSqlKind};
use super::{BindingState, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    BinaryOperator, BindingPattern, CallExpression, Declaration, Expression, Statement,
    VariableDeclaration, VariableDeclarator,
};

pub(super) fn record_statements(statements: &[Statement<'_>], visitor: &mut ScopeVisitor<'_>) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                record_variable_declaration(declaration, visitor);
            }
            Statement::ExportDeclaration(export) => {
                if let Declaration::VariableDeclaration(declaration) = &export.declaration {
                    record_variable_declaration(declaration, visitor);
                }
            }
            _ => {}
        }
    }
}

fn record_variable_declaration(
    declaration: &VariableDeclaration<'_>,
    visitor: &mut ScopeVisitor<'_>,
) {
    let is_const = declaration.kind == oxc_ast::ast::VariableDeclarationKind::Const;
    for declarator in &declaration.declarations {
        record_declarator(declarator, is_const, visitor);
    }
}

fn record_declarator(
    declarator: &VariableDeclarator<'_>,
    is_const: bool,
    visitor: &mut ScopeVisitor<'_>,
) {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return;
    };
    let Some(init) = &declarator.init else {
        return;
    };
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, ident.span.start as usize);
    let (sql, kind) = classify_init(init, is_const, visitor);
    if let Some(scope) = visitor.current_scope() {
        scope.insert(ident.name.to_string(), BindingState { sql, kind, line });
    }
}

pub(super) fn classify_init(
    expr: &Expression<'_>,
    is_const: bool,
    visitor: &ScopeVisitor<'_>,
) -> (Option<String>, EmbeddedSqlKind) {
    if let Some((text, kind)) = composed_sql(expr, visitor) {
        return if is_const {
            (Some(text), kind)
        } else {
            (Some(text), EmbeddedSqlKind::Dynamic)
        };
    }
    if interpolating_untrusted_tag(expr) {
        return (sql_text(expr), EmbeddedSqlKind::Dynamic);
    }
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => kind_for_const(literal.value.to_string(), is_const),
        Expression::TaggedTemplateExpression(_) => {
            kind_for_const(sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
            kind_for_const(sql_text(expr).unwrap_or_default(), is_const)
        }
        Expression::TemplateLiteral(_) => (sql_text(expr), EmbeddedSqlKind::Dynamic),
        Expression::CallExpression(_) => match resolve_chain(expr, visitor) {
            Some(text) if is_const => (Some(text), EmbeddedSqlKind::Composed),
            Some(text) => (Some(text), EmbeddedSqlKind::Dynamic),
            None => (None, EmbeddedSqlKind::Dynamic),
        },
        _ => (None, EmbeddedSqlKind::Dynamic),
    }
}

fn composed_sql(
    expr: &Expression<'_>,
    visitor: &ScopeVisitor<'_>,
) -> Option<(String, EmbeddedSqlKind)> {
    let Expression::BinaryExpression(binary) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    if binary.operator != BinaryOperator::Addition {
        return None;
    }
    let left = static_fragment(&binary.left, visitor)?;
    let right = static_fragment(&binary.right, visitor)?;
    Some((format!("{left}{right}"), EmbeddedSqlKind::Composed))
}

fn static_fragment(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => sql_text(expr),
        Expression::TaggedTemplateExpression(_) if interpolating_untrusted_tag(expr) => None,
        Expression::TaggedTemplateExpression(_) => sql_text(expr),
        Expression::BinaryExpression(_) => composed_sql(expr, visitor).map(|(text, _)| text),
        Expression::CallExpression(_) => resolve_chain(expr, visitor),
        _ => None,
    }
}

/// Resolves a fluent `.append()` chain or a call into a same-file
/// statically-composed function, per [`chain::resolve_expr`]. A callee name
/// shadowed by an in-scope parameter or nested local at this call site is
/// rejected rather than resolved against the same-named top-level
/// declaration; the top-level declaration's own binding (e.g. a const-bound
/// helper referencing itself) is not a shadow of itself.
fn resolve_chain(expr: &Expression<'_>, visitor: &ScopeVisitor<'_>) -> Option<String> {
    let mut lookup = |name: &str, _depth: u8| {
        if visitor.shadowed_locally(name) {
            return None;
        }
        visitor.functions.get(name)
    };
    chain::resolve_expr(expr, functions::MAX_RESOLVE_DEPTH, &mut lookup)
}

pub(super) fn executor_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    callee: String,
) -> EmbeddedSqlCall {
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, call.span.start as usize);
    let Some(argument) = first_call_argument(call) else {
        return EmbeddedSqlCall {
            line,
            callee,
            sql_text: None,
            kind: EmbeddedSqlKind::Dynamic,
            declaration_line: None,
        };
    };
    match unwrap_ts_wrappers(argument) {
        Expression::Identifier(ident) => {
            let binding = visitor.lookup(ident.name.as_str());
            EmbeddedSqlCall {
                line,
                callee,
                sql_text: binding.as_ref().and_then(|binding| binding.sql.clone()),
                kind: binding
                    .as_ref()
                    .map(|binding| binding.kind)
                    .unwrap_or(EmbeddedSqlKind::Dynamic),
                declaration_line: binding.map(|binding| binding.line),
            }
        }
        _ => {
            let (sql, kind) = classify_init(argument, true, visitor);
            EmbeddedSqlCall {
                line,
                callee,
                sql_text: sql,
                kind: if kind == EmbeddedSqlKind::ImmutableLocal {
                    EmbeddedSqlKind::Inline
                } else {
                    kind
                },
                declaration_line: None,
            }
        }
    }
}
