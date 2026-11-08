use super::{is_placeholder_ident_at, Resolver, Sourced};
use crate::codebase::postgres::idents::{unwrap_expr, visit_child_exprs};
use crate::codebase::postgres::statements::row_variant::is_row_variant;
use crate::codebase::postgres::statements::{SqlBareRead, SqlPinSource, SqlQualifiedRead};
use sqlparser::ast::{Expr, FunctionArguments};
use std::collections::BTreeSet;

/// What an expression refers to among the FROM items.
#[derive(Default)]
struct Refs {
    items: BTreeSet<usize>,
    /// A column the FROM items cannot explain: an outer reference or an ambiguous bare name.
    unknown: bool,
    /// Bare columns that subqueries of the expression read, if no table of theirs has them.
    reads: Vec<SqlBareRead>,
    qualified_reads: Vec<SqlQualifiedRead>,
}

impl Resolver {
    fn refs(&self, expr: &Expr, found: &mut Refs) {
        match unwrap_expr(expr) {
            // A scalar subquery is a value the statement does not size, unless it reads the row.
            Expr::Subquery(query) => {
                let reads = self.reads(query);
                found.unknown |= reads.certain;
                found.reads.extend(reads.bare);
                found.qualified_reads.extend(reads.qualified);
            }
            Expr::Exists { .. } => {}
            Expr::CompoundFieldAccess { root, access_chain } => {
                if let Some((base, indexes)) =
                    super::super::array::argument_base(root, access_chain)
                {
                    self.refs(&base, found);
                    for index in indexes {
                        self.refs(index, found);
                    }
                } else {
                    visit_child_exprs(expr, &mut |child| self.refs(child, found));
                }
            }
            // An explicit collation changes what `=` matches, whatever the key's own collation
            // (`email = $1 COLLATE "case_insensitive"`): the value fixes no row.
            Expr::Collate { .. } => found.unknown = true,
            // A function that differs per row is not a fixed value, and `ARRAY(SELECT …)` is the
            // rows of a query, not a value the caller sized.
            Expr::Function(function)
                if is_row_variant(&function.name)
                    || matches!(function.args, FunctionArguments::Subquery(_)) =>
            {
                found.unknown = true
            }
            // A bind, whether `$1` or an interpolation recovered from a template literal.
            Expr::Identifier(ident)
                if is_placeholder_ident_at(ident, self.positions.as_deref()) => {}
            expr @ (Expr::Identifier(_) | Expr::CompoundIdentifier(_)) => match self.column(expr) {
                Some((item, column)) => {
                    if !self.caller_outputs[item].contains(&column) {
                        found.items.insert(item);
                    }
                }
                None => found.unknown = true,
            },
            other => visit_child_exprs(other, &mut |child| self.refs(child, found)),
        }
    }

    pub(in super::super) fn source(&self, value: &Expr, pinned: usize) -> Option<Sourced> {
        self.source_excluding(value, Some(pinned))
    }

    pub(in super::super) fn correlated_source(&self, value: &Expr) -> Option<Sourced> {
        let Expr::CompoundIdentifier(parts) = unwrap_expr(value) else {
            return None;
        };
        if parts.len() != 2 || self.column(value).is_some() {
            return None;
        }
        Some(Sourced {
            source: SqlPinSource::Value,
            reads: Vec::new(),
            qualified_reads: vec![SqlQualifiedRead {
                qualifier: parts[0].to_string(),
                scopes: Vec::new(),
            }],
        })
    }

    /// Stored-array syntax retains self-owned dependencies without finite-key credit.
    pub(in super::super) fn stored_source(&self, value: &Expr) -> Option<Sourced> {
        self.source_excluding(value, None)
    }

    fn source_excluding(&self, value: &Expr, pinned: Option<usize>) -> Option<Sourced> {
        let mut found = Refs::default();
        self.refs(value, &mut found);
        if found.unknown || pinned.is_some_and(|pinned| found.items.contains(&pinned)) {
            return None;
        }
        let source = if found.items.is_empty() {
            SqlPinSource::Value
        } else {
            SqlPinSource::Items(found.items.into_iter().collect())
        };
        Some(Sourced {
            source,
            reads: found.reads,
            qualified_reads: found.qualified_reads,
        })
    }
}
