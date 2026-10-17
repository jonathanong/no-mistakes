//! Calls to helpers imported from other modules.
//!
//! The walk sees only this file, so it composes a private hole where the
//! helper's SQL would go and records the call's arguments. The projection step
//! later resolves the import and fills the hole (see `helpers`).

use super::super::helpers::split::{compose, plain, Marker};
use super::super::helpers::{sentinel, ImportedSite, SiteArg};
use super::resolve::compose::static_fragment;
use super::resolve::functions::provenance::RecoveredSql;
use super::ScopeVisitor;
use crate::codebase::postgres::embedded::{unpublished_sql_text, EmbeddedSqlKind};
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
};

/// Local names of named and default value imports. Namespace imports are
/// excluded: `ns.helper()` is a member call, which stays unrecovered.
pub(super) fn value_import_names(program: &Program<'_>) -> FxHashSet<String> {
    let mut names = FxHashSet::default();
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        if import.import_kind == ImportOrExportKind::Type {
            continue;
        }
        for specifier in import.specifiers.iter().flatten() {
            match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.import_kind != ImportOrExportKind::Type =>
                {
                    names.insert(named.local.name.to_string());
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                    names.insert(default.local.name.to_string());
                }
                _ => {}
            }
        }
    }
    names
}

impl ScopeVisitor<'_> {
    /// Whether `name` currently refers to a value import rather than a local.
    pub(super) fn is_imported_helper(&self, name: &str) -> bool {
        self.imported_names.contains(name) && !self.shadowed_locally(name)
    }
}

/// Whether the call hands a SQL builder to an imported helper, which may then
/// return that same (mutable) builder.
pub(super) fn passes_builder(visitor: &ScopeVisitor<'_>, call: &CallExpression<'_>) -> bool {
    classify(visitor, call).is_some_and(|(_, builder)| builder.is_some())
}

/// The SQL a call to imported helper `name` contributes: a hole, preceded by
/// the builder's own text when the helper receives one.
pub(in crate::codebase::postgres::embedded::walk) fn recover_call(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    name: &str,
) -> Option<RecoveredSql> {
    if call.optional || !visitor.imported_names.contains(name) || visitor.shadowed_locally(name) {
        return None;
    }
    let (args, builder) = classify(visitor, call)?;
    let line = byte_offset_to_line(visitor.source, call.span.start as usize);
    let index = visitor.imported_sites.borrow_mut().intern(
        call.span.start,
        ImportedSite {
            local: name.to_string(),
            args,
        },
    );
    let hole = plain(
        sentinel::encode(index),
        Marker {
            line,
            path: None,
            origin: call.span.start,
        },
    );
    let Some(argument) = builder else {
        return Some(hole);
    };
    let (sql, positions, origin) = visitor.functions.argument_base(argument, call, visitor)?;
    let base = RecoveredSql {
        origins: vec![call.span.start; sql.len()],
        sql,
        line: origin,
        positions,
    };
    compose([base, hole])
}

/// Every argument must be a value the projection can reason about; at most one
/// may be a SQL builder.
fn classify<'a>(
    visitor: &ScopeVisitor<'_>,
    call: &'a CallExpression<'a>,
) -> Option<(Vec<SiteArg>, Option<&'a Expression<'a>>)> {
    let mut args = Vec::with_capacity(call.arguments.len());
    let mut builder = None;
    for argument in &call.arguments {
        let expression = argument.as_expression()?;
        let arg = match unwrap_ts_wrappers(expression) {
            Expression::StringLiteral(literal) => SiteArg::Literal(literal.value.to_string()),
            Expression::TemplateLiteral(template) if template.expressions.is_empty() => {
                SiteArg::Literal(unpublished_sql_text(expression)?)
            }
            Expression::NumericLiteral(_)
            | Expression::BooleanLiteral(_)
            | Expression::NullLiteral(_) => SiteArg::Scalar,
            Expression::Identifier(ident) => {
                let known = visitor.lookup(ident.name.as_str());
                if known.is_some_and(|binding| {
                    binding.sql_builder
                        && binding.sql.is_some()
                        && binding.kind != EmbeddedSqlKind::Dynamic
                }) {
                    SiteArg::Builder
                } else {
                    SiteArg::Scalar
                }
            }
            Expression::TaggedTemplateExpression(_)
                if static_fragment(expression, visitor).is_some() =>
            {
                SiteArg::Builder
            }
            _ => return None,
        };
        if arg == SiteArg::Builder && builder.replace(expression).is_some() {
            return None;
        }
        args.push(arg);
    }
    Some((args, builder))
}
