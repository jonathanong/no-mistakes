mod fragment;

use super::provenance::RecoveredSql;
use super::{shadows_param, Resolvable};
use crate::codebase::postgres::embedded::tags::SqlTagNames;
use crate::fx::FxHashMap;
use fragment::{apply_append_statement, resolve_fragment};
use oxc_ast::ast::{
    BindingPattern, Statement, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};

/// Straight-line same-file helper bodies: `const`/`let` SQL inits, `.append`
/// mutations, and a final `return`. Control flow, assignment, and parameter
/// identifiers fail closed rather than guessing SQL.
pub(super) fn resolve(
    resolvable: &Resolvable<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&oxc_ast::ast::CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &SqlTagNames,
) -> Option<RecoveredSql> {
    let mut locals = FxHashMap::default();
    let mut statements = resolvable.body.statements.iter().peekable();
    while let Some(statement) = statements.next() {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration) => {
                bind_declaration(
                    declaration,
                    resolvable,
                    source,
                    depth,
                    lookup,
                    is_shadowed,
                    imported_sql_tags,
                    &mut locals,
                )?;
            }
            Statement::ExpressionStatement(statement) => {
                apply_append_statement(
                    statement,
                    resolvable,
                    source,
                    depth,
                    lookup,
                    is_shadowed,
                    imported_sql_tags,
                    &mut locals,
                )?;
            }
            Statement::ReturnStatement(ret) => {
                if statements.peek().is_some() {
                    return None;
                }
                return resolve_fragment(
                    ret.argument.as_ref()?,
                    resolvable,
                    source,
                    depth,
                    lookup,
                    is_shadowed,
                    imported_sql_tags,
                    &locals,
                );
            }
            _ => return None,
        }
    }
    None
}

fn bind_declaration(
    declaration: &VariableDeclaration<'_>,
    resolvable: &Resolvable<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&oxc_ast::ast::CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &SqlTagNames,
    locals: &mut FxHashMap<String, RecoveredSql>,
) -> Option<()> {
    if !matches!(
        declaration.kind,
        VariableDeclarationKind::Const | VariableDeclarationKind::Let
    ) {
        return None;
    }
    for declarator in &declaration.declarations {
        bind_declarator(
            declarator,
            resolvable,
            source,
            depth,
            lookup,
            is_shadowed,
            imported_sql_tags,
            locals,
        )?;
    }
    Some(())
}

fn bind_declarator(
    declarator: &VariableDeclarator<'_>,
    resolvable: &Resolvable<'_>,
    source: &str,
    depth: u8,
    lookup: &mut impl FnMut(&oxc_ast::ast::CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &SqlTagNames,
    locals: &mut FxHashMap<String, RecoveredSql>,
) -> Option<()> {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return None;
    };
    let name = ident.name.as_str();
    if shadows_param(resolvable, name) || locals.contains_key(name) {
        return None;
    }
    let text = resolve_fragment(
        declarator.init.as_ref()?,
        resolvable,
        source,
        depth,
        lookup,
        is_shadowed,
        imported_sql_tags,
        locals,
    )?;
    locals.insert(name.to_string(), text);
    Some(())
}
