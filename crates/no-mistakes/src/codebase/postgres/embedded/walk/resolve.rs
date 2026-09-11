mod append;
mod chain;
mod compose;
mod functions;
mod loops;

pub(super) use append::apply_append;
pub(in crate::codebase::postgres::embedded::walk) use compose::{
    appended_builder_fragment, builder_fragment, is_builder_append,
};
pub(super) use functions::LocalFunctions;
pub(super) use loops::{bind_for_statement_left, enter_classic_for, leave_classic_for};

use super::super::{first_call_argument, EmbeddedSqlCall, EmbeddedSqlKind};
use super::{BindingState, ScopeVisitor};
use crate::codebase::ts_source::unwrap_ts_wrappers;
use compose::classify_init;
use oxc_ast::ast::{
    BindingPattern, CallExpression, Declaration, Expression, Function, Statement,
    VariableDeclaration, VariableDeclarator,
};

/// Every name a parameter or declarator's binding pattern introduces,
/// however deeply destructured — `x`, `{ a: x }`, `[x]`, `{ x = 1 }`, and any
/// nesting or combination of those, plus rest elements. A shadow check that
/// only handled a bare `BindingIdentifier` would miss a destructured
/// parameter shadowing a same-named top-level helper (`function f({ safe })`
/// shadows a top-level `safe`), letting `resolve_chain`/`resolve_named`
/// wrongly resolve calls through it.
pub(super) fn for_each_bound_name<'a>(
    pattern: &BindingPattern<'a>,
    on_name: &mut impl FnMut(&'a str),
) {
    match pattern {
        BindingPattern::BindingIdentifier(ident) => on_name(ident.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                for_each_bound_name(&property.value, on_name);
            }
            if let Some(rest) = &object.rest {
                for_each_bound_name(&rest.argument, on_name);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                for_each_bound_name(element, on_name);
            }
            if let Some(rest) = &array.rest {
                for_each_bound_name(&rest.argument, on_name);
            }
        }
        BindingPattern::AssignmentPattern(assignment) => {
            for_each_bound_name(&assignment.left, on_name);
        }
    }
}

pub(super) fn record_statements(statements: &[Statement<'_>], visitor: &mut ScopeVisitor<'_>) {
    for statement in statements {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                record_variable_declaration(declaration, visitor);
            }
            Statement::FunctionDeclaration(function) => {
                record_function_declaration(function, visitor);
            }
            Statement::ClassDeclaration(class) => {
                record_nested_type_name(class.id.as_ref().map(|id| id.name.as_str()), visitor);
            }
            Statement::ExportDeclaration(export) => match &export.declaration {
                Declaration::VariableDeclaration(declaration) => {
                    record_variable_declaration(declaration, visitor);
                }
                Declaration::FunctionDeclaration(function) => {
                    record_function_declaration(function, visitor);
                }
                Declaration::ClassDeclaration(class) => {
                    record_nested_type_name(class.id.as_ref().map(|id| id.name.as_str()), visitor);
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/// Binds a nested function declaration's own name into its enclosing scope,
/// as a shadow marker only — `LocalFunctions` never collects a nested
/// declaration's body (see its own doc comment), so this exists solely to
/// make `shadowed_locally` recognize that the name no longer refers to a
/// same-named top-level helper within this scope. A top-level declaration
/// is skipped: it lands in the program's own outermost scope, which
/// `shadowed_locally` always excludes, so recording it there would only
/// risk clobbering a same-named top-level `const` binding's own already
/// classified `BindingState` (JS forbids the collision within one real
/// scope, but the parser doesn't enforce that, and existing fixtures rely
/// on a same-named top-level helper never shadowing itself).
fn record_function_declaration(function: &Function<'_>, visitor: &mut ScopeVisitor<'_>) {
    if visitor.scopes.len() <= 1 {
        return;
    }
    let Some(id) = &function.id else { return };
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, id.span.start as usize);
    if let Some(scope) = visitor.current_scope() {
        scope.insert(
            id.name.to_string(),
            BindingState {
                sql: None,
                kind: EmbeddedSqlKind::Dynamic,
                line,
                sql_builder: false,
            },
        );
    }
}

fn record_nested_type_name(name: Option<&str>, visitor: &mut ScopeVisitor<'_>) {
    if visitor.scopes.len() <= 1 {
        return;
    }
    if let Some(name) = name {
        visitor.bind_self_name(name);
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
        // Destructuring has no single SQL init to classify; bind every name
        // as a shadow so a nested `const { tag } = …` cannot keep a trusted
        // imported tag alias.
        visitor.bind_param(&declarator.id, false);
        return;
    };
    let Some(init) = &declarator.init else {
        return;
    };
    let line =
        crate::codebase::ts_source::byte_offset_to_line(visitor.source, ident.span.start as usize);
    let (sql, kind) = classify_init(init, is_const, visitor);
    if let Some(scope) = visitor.current_scope() {
        scope.insert(
            ident.name.to_string(),
            BindingState {
                sql_builder: sql.is_some()
                    && matches!(
                        kind,
                        EmbeddedSqlKind::ImmutableLocal | EmbeddedSqlKind::Composed
                    ),
                sql,
                kind,
                line,
            },
        );
    }
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
                sql_text: binding.as_ref().and_then(|binding| {
                    binding
                        .sql
                        .clone()
                        .map(super::super::placeholders::publish_placeholders)
                }),
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
                sql_text: sql.map(super::super::placeholders::publish_placeholders),
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
