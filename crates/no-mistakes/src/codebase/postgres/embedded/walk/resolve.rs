use super::super::{first_call_argument, sql_text, EmbeddedSqlCall, EmbeddedSqlKind};
use super::{BindingState, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{
    Argument, BinaryOperator, BindingPattern, CallExpression, Declaration, Expression, Statement,
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
    let (sql, kind) = classify_init(init, is_const);
    if let Some(scope) = visitor.current_scope() {
        scope.insert(ident.name.to_string(), BindingState { sql, kind, line });
    }
}

pub(super) fn classify_init(
    expr: &Expression<'_>,
    is_const: bool,
) -> (Option<String>, EmbeddedSqlKind) {
    if let Some((text, kind)) = composed_sql(expr) {
        return if is_const {
            (Some(text), kind)
        } else {
            (Some(text), EmbeddedSqlKind::Dynamic)
        };
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
        _ => (None, EmbeddedSqlKind::Dynamic),
    }
}

fn kind_for_const(sql: String, is_const: bool) -> (Option<String>, EmbeddedSqlKind) {
    if is_const {
        (Some(sql), EmbeddedSqlKind::ImmutableLocal)
    } else {
        (Some(sql), EmbeddedSqlKind::Dynamic)
    }
}

fn composed_sql(expr: &Expression<'_>) -> Option<(String, EmbeddedSqlKind)> {
    let Expression::BinaryExpression(binary) = unwrap_ts_wrappers(expr) else {
        return None;
    };
    if binary.operator != BinaryOperator::Addition {
        return None;
    }
    let left = static_fragment(&binary.left)?;
    let right = static_fragment(&binary.right)?;
    Some((format!("{left}{right}"), EmbeddedSqlKind::Composed))
}

fn static_fragment(expr: &Expression<'_>) -> Option<String> {
    match unwrap_ts_wrappers(expr) {
        Expression::StringLiteral(literal) => Some(literal.value.to_string()),
        Expression::TemplateLiteral(template) if template.expressions.is_empty() => sql_text(expr),
        Expression::TaggedTemplateExpression(_) => sql_text(expr),
        Expression::BinaryExpression(_) => composed_sql(expr).map(|(text, _)| text),
        _ => None,
    }
}

pub(super) fn apply_append(visitor: &mut ScopeVisitor<'_>, call: &CallExpression<'_>) {
    let Expression::StaticMemberExpression(member) = unwrap_ts_wrappers(&call.callee) else {
        return;
    };
    if member.property.name != "append" {
        return;
    }
    let Expression::Identifier(ident) = unwrap_ts_wrappers(&member.object) else {
        return;
    };
    let Some(arg) = first_static_arg(call) else {
        visitor.mark_dynamic(ident.name.as_str());
        return;
    };
    if visitor.control_depth > 0 {
        visitor.mark_dynamic(ident.name.as_str());
        return;
    }
    for scope in visitor.scopes.iter_mut().rev() {
        if let Some(binding) = scope.get_mut(ident.name.as_str()) {
            match (&binding.sql, binding.kind) {
                (Some(sql), EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed) => {
                    binding.sql = Some(format!("{sql}{arg}"));
                    binding.kind = EmbeddedSqlKind::Composed;
                }
                _ => {
                    binding.kind = EmbeddedSqlKind::Dynamic;
                    binding.sql = None;
                }
            }
            return;
        }
    }
}

fn first_static_arg(call: &CallExpression<'_>) -> Option<String> {
    let Argument::SpreadElement(_) = call.arguments.first()? else {
        let expr = call.arguments.first()?.as_expression()?;
        return static_fragment(expr);
    };
    None
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
            let (sql, kind) = classify_init(argument, true);
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
