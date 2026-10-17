//! Helpers that return one untagged template whose interpolations are their
//! own parameters, e.g. `` (parent: string) => `(${parent} IS NULL)` ``.
//!
//! The template is summarized once per file as structured data. A call whose
//! every argument is a string literal splices the literals in as SQL text.

use super::split::{compose, plain, split};
use crate::codebase::postgres::embedded::placeholders::INTERNAL_PLACEHOLDER;
use crate::codebase::postgres::embedded::unpublished_sql_text;
use crate::codebase::postgres::embedded::walk::resolve::functions::provenance::RecoveredSql;
use crate::codebase::ts_source::unwrap_ts_wrappers;
use oxc_ast::ast::{BindingPattern, Expression, FormalParameters};
use std::ops::Range;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct TemplateHelper {
    /// The whole template, each `${param}` kept as an internal placeholder.
    sql: RecoveredSql,
    /// Parameter index behind each placeholder, in text order.
    slots: Vec<usize>,
    parameters: usize,
}

impl TemplateHelper {
    /// Splices one string literal per parameter. `None` for any other call shape.
    pub(crate) fn apply(&self, literals: &[&str]) -> Option<RecoveredSql> {
        if literals.len() != self.parameters || !literals.iter().all(|text| splice_safe(text)) {
            return None;
        }
        // `summarize` made exactly one placeholder per slot, in the same order.
        let markers = placeholder_ranges(&self.sql.sql);
        let (pieces, origins) = split(&self.sql, &markers);
        let mut parts = Vec::with_capacity(pieces.len() * 2);
        let mut pieces = pieces.into_iter();
        parts.extend(pieces.next());
        for ((slot, marker), piece) in self.slots.iter().zip(origins).zip(pieces) {
            parts.push(plain(literals[*slot].to_string(), marker));
            parts.push(piece);
        }
        compose(parts)
    }
}

/// A literal is spliced verbatim, so it must keep the physical line map exact
/// and must not forge an internal marker.
fn splice_safe(text: &str) -> bool {
    !text.contains(['\n', '\r', '\u{10FFFE}', '\u{10FFFF}'])
}

fn placeholder_ranges(sql: &str) -> Vec<Range<usize>> {
    sql.match_indices(INTERNAL_PLACEHOLDER)
        .map(|(start, marker)| {
            let digits_start = start + marker.len();
            let digits = sql[digits_start..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            start..digits_start + digits
        })
        .collect()
}

/// Summarizes one callable, or `None` when it is not a literal-identifier template.
pub(crate) fn summarize(
    params: &FormalParameters<'_>,
    returned: &Expression<'_>,
    source: &str,
) -> Option<TemplateHelper> {
    let names = parameter_names(params)?;
    let Expression::TemplateLiteral(template) = unwrap_ts_wrappers(returned) else {
        return None;
    };
    if template.expressions.is_empty() {
        return None;
    }
    let slots = template
        .expressions
        .iter()
        .map(|expression| match unwrap_ts_wrappers(expression) {
            Expression::Identifier(ident) => {
                names.iter().position(|name| *name == ident.name.as_str())
            }
            _ => None,
        })
        .collect::<Option<Vec<_>>>()?;
    let sql = unpublished_sql_text(returned)?;
    Some(TemplateHelper {
        sql: RecoveredSql::literal(returned, source, sql),
        slots,
        parameters: names.len(),
    })
}

/// Plain identifiers only: no default, rest, optional or destructured parameter.
fn parameter_names<'a>(params: &'a FormalParameters<'_>) -> Option<Vec<&'a str>> {
    if params.rest.is_some() {
        return None;
    }
    let mut names = Vec::with_capacity(params.items.len());
    for param in &params.items {
        let BindingPattern::BindingIdentifier(ident) = &param.pattern else {
            return None;
        };
        if param.initializer.is_some() || param.optional || names.contains(&ident.name.as_str()) {
            return None;
        }
        names.push(ident.name.as_str());
    }
    Some(names)
}
