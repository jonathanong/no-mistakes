//! The bounded-iteration shapes: `literal-limit` and `keyset-only-sweep`.
use super::scan::finding;
use super::{BannedShapes, RuleFinding, RULE_ID};
use crate::codebase::postgres::idents::unwrap_expr;
use crate::codebase::postgres::{
    parse_postgres_expression, SqlConjunctFact, SqlCursorBound, SqlLimitValue,
    SqlStatementFileFacts,
};
use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ShapeOptions {
    literal_limit: LiteralLimit,
    keyset_only_sweep: KeysetOnlySweep,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct LiteralLimit {
    allowed_values: Option<Vec<i64>>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct KeysetOnlySweep {
    non_selective_predicates: Vec<String>,
    ignore_tables: Vec<String>,
}

pub(crate) struct IterationOptions {
    allowed_limits: Vec<u64>,
    /// Normalized conjuncts that do not narrow a walk.
    non_selective: Vec<String>,
    /// Decoded SQL name parts of tables that may be walked whole.
    ignore_tables: Vec<Vec<String>>,
}

pub(super) fn compile(options: &ShapeOptions) -> Result<IterationOptions> {
    let mut allowed_limits = Vec::new();
    for value in options
        .literal_limit
        .allowed_values
        .as_deref()
        .unwrap_or(&[1])
    {
        let Ok(value) = u64::try_from(*value) else {
            bail!(
                "{RULE_ID} option shapeOptions.literalLimit.allowedValues: negative value {value}"
            );
        };
        allowed_limits.push(value);
    }
    let sweep = &options.keyset_only_sweep;
    let non_selective = names(
        &sweep.non_selective_predicates,
        "nonSelectivePredicates",
        normalized_predicate,
    )?;
    let ignore_tables = names(&sweep.ignore_tables, "ignoreTables", str::to_string)?
        .iter()
        .map(|name| crate::codebase::postgres::decoded_parts(name))
        .collect();
    Ok(IterationOptions {
        allowed_limits,
        non_selective,
        ignore_tables,
    })
}

/// A conjunct as the statement pass renders it, so `a=1` and `a = 1` compare equal.
fn normalized_predicate(text: &str) -> String {
    let rendered = parse_postgres_expression(text)
        .map(|expression| unwrap_expr(&expression).to_string())
        .unwrap_or_else(|| text.to_string());
    crate::codebase::postgres::predicate_normalization::normalize(&rendered)
}

fn names(values: &[String], option: &str, shape: impl Fn(&str) -> String) -> Result<Vec<String>> {
    values
        .iter()
        .map(|value| {
            if value.trim().is_empty() {
                bail!("{RULE_ID} option shapeOptions.keysetOnlySweep.{option}: empty string");
            }
            Ok(shape(value.trim()))
        })
        .collect()
}

impl IterationOptions {
    /// An entry names the table as `schema.table` or by its last name part alone. A dot inside a
    /// quoted name belongs to that part: `items` does not match `"work.items"`.
    fn ignored(&self, parts: &[String]) -> bool {
        self.ignore_tables
            .iter()
            .any(|entry| entry == parts || (entry.len() == 1 && entry.last() == parts.last()))
    }
}

/// A conjunct that compares ordered columns with a bind: the cursor of a keyset walk.
fn is_cursor(conjunct: &SqlConjunctFact, order_columns: &[String]) -> bool {
    !conjunct.cursor_columns.is_empty()
        && conjunct
            .cursor_columns
            .iter()
            .all(|column| order_columns.contains(column))
}

pub(super) fn findings(
    file: &str,
    facts: &SqlStatementFileFacts,
    shapes: &BannedShapes,
    options: &IterationOptions,
    line_at: impl Fn(usize) -> usize,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if shapes.literal_limit {
        for limit in &facts.limit_uses {
            let SqlLimitValue::Literal(value) = limit.value else {
                continue;
            };
            if !options.allowed_limits.contains(&value) {
                findings.push(finding(
                    file,
                    line_at(limit.line),
                    &format!("LIMIT {value} is a literal batch size that cannot be tuned without a deploy; bind it (LIMIT $1) or list {value} in shapeOptions.literalLimit.allowedValues"),
                    "literal-limit",
                ));
            }
        }
    }
    if shapes.keyset_only_sweep {
        for sweep in &facts.sweeps {
            let cursors = |bound| {
                sweep.conjuncts.iter().any(|conjunct| {
                    conjunct.cursor_bound == Some(bound)
                        && !conjunct.cursor_optional
                        && is_cursor(conjunct, &sweep.order_columns)
                })
            };
            // A cursor bounds one side of the walk; a lower and an upper bound together are
            // a window, which narrows it. An optional bound (a NULL bind switches it off) does
            // not count: with both binds NULL the walk is whole.
            let windowed = cursors(SqlCursorBound::Lower) && cursors(SqlCursorBound::Upper);
            let selective = windowed
                || sweep.conjuncts.iter().any(|conjunct| {
                    !is_cursor(conjunct, &sweep.order_columns)
                        && !conjunct.constant_true
                        && !conjunct.bind_guard
                        && !options.non_selective.contains(&conjunct.text)
                });
            if !selective && !options.ignored(&sweep.table_parts) {
                findings.push(finding(
                    file,
                    line_at(sweep.line),
                    &format!("walks every row of {} in {} order with nothing else selective; choose rows that need work (a work-item row, a dirty marker, a due timestamp or a parent id) or list the table in shapeOptions.keysetOnlySweep.ignoreTables", sweep.table, sweep.order_columns.join(", ")),
                    "keyset-only-sweep",
                ));
            }
        }
    }
    findings
}
