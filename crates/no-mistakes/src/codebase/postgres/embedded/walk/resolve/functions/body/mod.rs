mod fragment;

use super::{shadows_param, Resolvable};
use fragment::{apply_append_statement, resolve_fragment};
use oxc_ast::ast::{
    BindingPattern, Statement, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};
use std::collections::{HashMap, HashSet};

/// Straight-line same-file helper bodies: `const`/`let` SQL inits, `.append`
/// mutations, and a final `return`. Control flow, assignment, and parameter
/// identifiers fail closed rather than guessing SQL.
#[inline(never)]
pub(super) fn resolve(
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
) -> Option<String> {
    let mut locals = HashMap::new();
    let mut statements = resolvable.body.statements.iter().peekable();
    while let Some(statement) = statements.next() {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration) => {
                bind_declaration(
                    declaration,
                    resolvable,
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

#[inline(never)]
fn bind_declaration(
    declaration: &VariableDeclaration<'_>,
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
    locals: &mut HashMap<String, String>,
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
            depth,
            lookup,
            is_shadowed,
            imported_sql_tags,
            locals,
        )?;
    }
    Some(())
}

#[inline(never)]
fn bind_declarator(
    declarator: &VariableDeclarator<'_>,
    resolvable: &Resolvable<'_>,
    depth: u8,
    lookup: &mut impl FnMut(&str, u8) -> Option<String>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    imported_sql_tags: &HashSet<String>,
    locals: &mut HashMap<String, String>,
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
        depth,
        lookup,
        is_shadowed,
        imported_sql_tags,
        locals,
    )?;
    locals.insert(name.to_string(), text);
    Some(())
}
