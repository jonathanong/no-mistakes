//! Calls to helpers imported from other modules.
//!
//! The walk sees only this file, so it composes a private hole where the
//! helper's SQL would go and records the call's arguments. The projection step
//! later resolves the import and fills the hole (see `helpers`).

use super::super::helpers::split::{compose, plain, Marker};
use super::super::helpers::{sentinel, ImportedSite, SiteArg};
use super::resolve::functions::provenance::RecoveredSql;
use super::ScopeVisitor;
use crate::codebase::ts_source::{byte_offset_to_line, unwrap_ts_wrappers};
use crate::fx::FxHashSet;
use oxc_ast::ast::{
    CallExpression, Expression, ImportDeclarationSpecifier, ImportOrExportKind, Program, Statement,
};
mod args;
use args::classify;

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
    /// Whether a call to `name` becomes a hole the projection fills: a value
    /// import, or a same-file helper that only a lowered program can recover
    /// (the established same-file machinery keeps precedence).
    pub(super) fn is_hole_helper(&self, name: &str) -> bool {
        if self.shadowed_locally(name) {
            return false;
        }
        // The SQL tag itself is an import too, but it is never a helper.
        if self.imported_names.contains(name) {
            return !self.functions.imported_sql_tags().contains(name);
        }
        self.functions.has_program(name)
            && !self.functions.has_static_helper(name)
            && !self.functions.is_parameter_builder(name)
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
    if call.optional || !visitor.is_hole_helper(name) {
        return None;
    }
    let (args, builder) = classify(visitor, call)?;
    builders_accepted(visitor, name, &args)?;
    let hole = site_hole(visitor, call, name, args, true);
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

/// `helper(builder, ..)` on an imported helper, when `builder` is a tracked
/// local: the builder it mutates and the hole its appended fragments fill.
pub(in crate::codebase::postgres::embedded::walk) fn builder_statement(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
) -> Option<(String, RecoveredSql)> {
    let Expression::Identifier(callee) = unwrap_ts_wrappers(&call.callee) else {
        return None;
    };
    let name = callee.name.as_str();
    // An executor is an import too, but it consumes the builder rather than extending it.
    if call.optional || !visitor.is_hole_helper(name) || visitor.bindings.contains(name) {
        return None;
    }
    let (args, builder) = classify(visitor, call)?;
    builders_accepted(visitor, name, &args)?;
    let Expression::Identifier(receiver) = unwrap_ts_wrappers(builder?) else {
        return None;
    };
    // A same-file helper keeps the established per-call effect analysis unless
    // the call is straight-line in the builder's own function.
    let straight_line = visitor.control_depth == 0
        && visitor.loop_depth == 0
        && !visitor.append_crosses_function(receiver.name.as_str());
    if !visitor.imported_names.contains(name) && !straight_line {
        return None;
    }
    Some((
        receiver.name.to_string(),
        site_hole(
            visitor,
            call,
            name,
            args,
            !visitor.statement_calls.contains(&call.span.start),
        ),
    ))
}

/// A same-file helper takes a caller's builder only at a `SQLStatement`
/// parameter; an imported one is checked where its program runs.
fn builders_accepted(visitor: &ScopeVisitor<'_>, name: &str, args: &[SiteArg]) -> Option<()> {
    if visitor.imported_names.contains(name) {
        return Some(());
    }
    args.iter()
        .enumerate()
        .all(|(index, arg)| {
            *arg != SiteArg::Builder || visitor.functions.program_takes_builder(name, index)
        })
        .then_some(())
}

fn site_hole(
    visitor: &ScopeVisitor<'_>,
    call: &CallExpression<'_>,
    name: &str,
    args: Vec<SiteArg>,
    value_used: bool,
) -> RecoveredSql {
    let line = byte_offset_to_line(visitor.source, call.span.start as usize);
    let index = visitor.imported_sites.borrow_mut().intern(
        call.span.start,
        ImportedSite {
            local: name.to_string(),
            args,
            value_used,
            line,
        },
    );
    plain(
        sentinel::encode(index),
        Marker {
            line,
            path: None,
            origin: call.span.start,
        },
    )
}
