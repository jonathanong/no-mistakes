use super::super::{CompiledOptions, PartitionExemption, RULE_ID};
use crate::codebase::postgres::{SchemaCatalog, SqlRelationPredicateFact, SqlSelectFact};
use crate::codebase::rules::RuleFinding;
use std::collections::BTreeSet;

pub(super) fn effective_columns(
    relations: &[SqlRelationPredicateFact],
    index: usize,
    catalog: Option<&SchemaCatalog>,
) -> BTreeSet<String> {
    let relation = &relations[index];
    let mut columns: BTreeSet<String> = relation.constrained_columns.iter().cloned().collect();
    let Some(catalog) = catalog else {
        return columns;
    };
    for column in &relation.unqualified_columns {
        let blocked = relations.iter().enumerate().any(|(other, peer)| {
            if other == index {
                return false;
            }
            match catalog.relation(&peer.table) {
                Some(info) => info
                    .columns
                    .iter()
                    .any(|column_info| column_info.name.eq_ignore_ascii_case(column)),
                None => true,
            }
        });
        if !blocked {
            columns.insert(column.clone());
        }
    }
    columns
}

pub(super) fn table_matches(left: &str, right: &str) -> bool {
    let right = right.trim().trim_matches('"');
    left.eq_ignore_ascii_case(right)
        || left
            .rsplit('.')
            .next()
            .is_some_and(|tail| tail.eq_ignore_ascii_case(right))
        || right
            .rsplit('.')
            .next()
            .is_some_and(|tail| left.eq_ignore_ascii_case(tail))
}

pub(super) fn exempt(table: &str, exemptions: &[PartitionExemption]) -> bool {
    exemptions
        .iter()
        .any(|entry| table_matches(table, &entry.table))
}

pub(super) fn textual_require(
    file: &str,
    select: &SqlSelectFact,
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for relation in &opts.relations {
        if !select
            .tables
            .iter()
            .any(|table| table.eq_ignore_ascii_case(&relation.table))
        {
            continue;
        }
        for required in &relation.require {
            if !contains_predicate(&select.predicate_sql, required) {
                findings.push(sql_finding(
                    file,
                    select.line.max(1),
                    &format!(
                        "queries against {} must include `{}`{}",
                        relation.table,
                        required,
                        equality_hint(&select.predicate_sql, required)
                    ),
                    Some(relation.table.as_str()),
                    None,
                ));
            }
        }
    }
    findings
}

/// An equality on a column that must be `IS NULL` selects exactly the
/// non-NULL rows the requirement excludes, so it never satisfies the
/// requirement. Explain the right fix instead of weakening the rule.
fn equality_hint(predicate_sql: &str, required: &str) -> String {
    let required = normalize(required);
    let Some(column) = required.strip_suffix(" is null") else {
        return String::new();
    };
    let sql = normalize(predicate_sql);
    if !(sql.contains(&format!("{column} = ")) || sql.contains(&format!("{column} in "))) {
        return String::new();
    }
    format!(
        "; an equality on `{column}` selects only rows where it is not NULL, which `{required}` excludes. If this query intentionally reads those rows, add `requireColumns: [{column}]` instead of `require`, or suppress with `no-mistakes-disable-next-line postgres-required-predicates` and a reason"
    )
}

fn contains_predicate(haystack: &str, needle: &str) -> bool {
    normalize(haystack).contains(&normalize(needle))
}

fn normalize(sql: &str) -> String {
    sql.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub(super) fn import_for(relation: &SqlRelationPredicateFact, column: &str) -> String {
    match &relation.alias {
        Some(alias) => format!("{alias}.{column}"),
        None => format!("{}.{column}", relation.table),
    }
}

pub(super) fn sql_finding(
    file: &str,
    line: usize,
    message: &str,
    target: Option<&str>,
    import: Option<String>,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import,
        target: target.map(ToOwned::to_owned),
    }
}
