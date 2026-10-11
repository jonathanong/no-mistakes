mod fragment;

use super::provenance::RecoveredSql;
use super::{shadows_param, Resolvable};
use crate::codebase::postgres::embedded::tags::SqlTagNames;
use crate::fx::FxHashMap;
use fragment::{apply_append_statement, resolve_fragment};
use oxc_ast::ast::{
    BindingPattern, Statement, VariableDeclaration, VariableDeclarationKind, VariableDeclarator,
};

struct BodySource<'a, 's> {
    resolvable: &'s Resolvable<'a>,
    source: &'s str,
    imported_sql_tags: &'s SqlTagNames,
}

/// Local names point to SQL values so aliases observe later in-place appends.
#[derive(Default)]
struct LocalSql {
    names: FxHashMap<String, usize>,
    values: Vec<RecoveredSql>,
}

impl LocalSql {
    fn get(&self, name: &str) -> Option<&RecoveredSql> {
        self.values.get(*self.names.get(name)?)
    }

    fn append(&mut self, name: &str, fragment: RecoveredSql) -> Option<()> {
        self.values
            .get_mut(*self.names.get(name)?)?
            .append(fragment);
        Some(())
    }
}

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
    let context = BodySource {
        resolvable,
        source,
        imported_sql_tags,
    };
    let mut locals = LocalSql::default();
    let mut statements = resolvable.body.statements.iter().peekable();
    while let Some(statement) = statements.next() {
        match statement {
            Statement::EmptyStatement(_) => {}
            Statement::VariableDeclaration(declaration) => {
                bind_declaration(
                    declaration,
                    &context,
                    depth,
                    lookup,
                    is_shadowed,
                    &mut locals,
                )?;
            }
            Statement::ExpressionStatement(statement) => {
                apply_append_statement(
                    statement,
                    &context,
                    depth,
                    lookup,
                    is_shadowed,
                    &mut locals,
                )?;
            }
            Statement::ReturnStatement(ret) => {
                if statements.peek().is_some() {
                    return None;
                }
                return resolve_fragment(
                    ret.argument.as_ref()?,
                    &context,
                    depth,
                    lookup,
                    is_shadowed,
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
    context: &BodySource<'_, '_>,
    depth: u8,
    lookup: &mut impl FnMut(&oxc_ast::ast::CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    locals: &mut LocalSql,
) -> Option<()> {
    if !matches!(
        declaration.kind,
        VariableDeclarationKind::Const | VariableDeclarationKind::Let
    ) {
        return None;
    }
    for declarator in &declaration.declarations {
        bind_declarator(declarator, context, depth, lookup, is_shadowed, locals)?;
    }
    Some(())
}

fn bind_declarator(
    declarator: &VariableDeclarator<'_>,
    context: &BodySource<'_, '_>,
    depth: u8,
    lookup: &mut impl FnMut(&oxc_ast::ast::CallExpression<'_>, &str, u8) -> Option<RecoveredSql>,
    is_shadowed: &mut impl FnMut(&str) -> bool,
    locals: &mut LocalSql,
) -> Option<()> {
    let BindingPattern::BindingIdentifier(ident) = &declarator.id else {
        return None;
    };
    let name = ident.name.as_str();
    if shadows_param(context.resolvable, name) || locals.names.contains_key(name) {
        return None;
    }
    let init = declarator.init.as_ref()?;
    if let oxc_ast::ast::Expression::Identifier(alias) =
        crate::codebase::ts_source::unwrap_ts_wrappers(init)
    {
        let index = *locals.names.get(alias.name.as_str())?;
        locals.names.insert(name.to_string(), index);
        return Some(());
    }
    let text = resolve_fragment(init, context, depth, lookup, is_shadowed, locals)?;
    let index = locals.values.len();
    locals.values.push(text);
    locals.names.insert(name.to_string(), index);
    Some(())
}
