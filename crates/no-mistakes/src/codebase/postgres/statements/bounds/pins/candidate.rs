use super::{Resolver, Sourced};
use crate::codebase::postgres::statements::SqlBoundPin;
use sqlparser::ast::Expr;

pub(super) fn equate(
    left: &Expr,
    right: &Expr,
    null_safe: bool,
    resolver: &Resolver,
    restricted: &[usize],
    out: &mut Vec<(usize, SqlBoundPin)>,
) {
    for (target, value) in [(left, right), (right, left)] {
        pin(target, null_safe, resolver, restricted, out, |item| {
            resolver.source(value, item)
        });
    }
}

pub(super) fn pin(
    target: &Expr,
    null_safe: bool,
    resolver: &Resolver,
    restricted: &[usize],
    out: &mut Vec<(usize, SqlBoundPin)>,
    mut source_for: impl FnMut(usize) -> Option<Sourced>,
) {
    let Some(column) = resolver.candidate_column(target) else {
        return;
    };
    for &item in restricted {
        if !resolver.is_table(item) {
            continue;
        }
        // A second unqualified column, a self-read, or a row-variant value cannot pin a key.
        let Some(sourced) = source_for(item) else {
            continue;
        };
        out.push((
            item,
            SqlBoundPin {
                column: column.clone(),
                unqualified_candidate: true,
                expression: None,
                source: sourced.source,
                null_safe,
                reads: sourced.reads,
                qualified_reads: sourced.qualified_reads,
            },
        ));
    }
}
