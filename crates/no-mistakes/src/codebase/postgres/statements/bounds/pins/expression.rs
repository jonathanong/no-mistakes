use super::Resolver;
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
        if resolver.column(target).is_some() {
            continue;
        }
        for &item in restricted {
            if !resolver.is_table(item) {
                continue;
            }
            let (Some(expression), Some(sourced)) = (
                resolver.canonical_for_item(target, item),
                resolver.source(value, item),
            ) else {
                continue;
            };
            out.push((
                item,
                SqlBoundPin {
                    alternative: None,
                    column: String::new(),
                    unqualified_candidate: false,
                    expression: Some(expression),
                    source: sourced.source,
                    null_safe,
                    reads: sourced.reads,
                    qualified_reads: sourced.qualified_reads,
                },
            ));
        }
    }
}
