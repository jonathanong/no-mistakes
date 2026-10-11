mod executor;
pub(super) use executor::executor_call;
mod provisional;
pub(super) use provisional::record_executor_call;
pub(super) mod append;
mod chain;
pub(super) mod compose;
pub(crate) mod functions;
mod loops;
pub(crate) mod vars;
pub(super) use vars::{hoist_vars, initialize_vars};

pub(super) use append::apply_append;
pub(in crate::codebase::postgres::embedded::walk) use compose::{
    appended_builder_fragment, builder_fragment, is_builder_append,
};
pub(super) use functions::LocalFunctions;
pub(super) use loops::{bind_for_statement_left, enter_classic_for, leave_classic_for};

use super::super::EmbeddedSqlKind;
use super::{BindingState, ScopeVisitor};
use compose::classify_init;
use oxc_ast::ast::{BindingPattern, Declaration, Function, Statement, VariableDeclaration};

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
                builder_identity: None,
                condition_key: None,
                variants: None,
                sql: None,
                kind: EmbeddedSqlKind::Dynamic,
                line,
                initialized: false,
                sql_builder: false,
                sql_source_positions: Vec::new(),
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
    // Function-scoped declarations are initialized at their actual visit.
    if declaration.kind == oxc_ast::ast::VariableDeclarationKind::Var {
        return;
    }
    let is_const = declaration.kind == oxc_ast::ast::VariableDeclarationKind::Const;
    for declarator in &declaration.declarations {
        vars::record_declarator(declarator, is_const, false, visitor);
    }
}
