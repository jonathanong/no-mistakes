use super::super::RULE_ID;
use super::substitute::substitute_target_columns;
use crate::codebase::postgres::{
    analyze_conflict_inserts, order_prefix_matches, CanonicalIndex, CanonicalOrderKey,
    ResolvedArbiter, SchemaCatalog, SqlConflictInsertFact, SqlConflictTarget, SqlInsertSourceShape,
};
use crate::codebase::rules::RuleFinding;

pub(super) fn findings_for_sql(
    file: &str,
    line: usize,
    sql: &str,
    catalog: &SchemaCatalog,
    fail_unanalyzable: bool,
) -> Vec<RuleFinding> {
    let inserts = match analyze_conflict_inserts(sql) {
        Ok(inserts) => inserts,
        Err(_) if fail_unanalyzable && contains_insert_conflict(sql) => {
            return vec![finding(
                file,
                line,
                "unanalyzable-sql",
                "keep INSERT ... ON CONFLICT SQL statically parseable so canonical ordering can be checked",
            )]
        }
        Err(_) => return Vec::new(),
    };
    inserts
        .into_iter()
        .filter(|insert| insert.source.multi_row)
        .filter_map(|insert| finding_for_insert(file, line, insert, catalog))
        .collect()
}

fn finding_for_insert(
    file: &str,
    line: usize,
    insert: SqlConflictInsertFact,
    catalog: &SchemaCatalog,
) -> Option<RuleFinding> {
    let (arbiter, target_is_ordered) = match &insert.target {
        SqlConflictTarget::Targetless => {
            return Some(finding(
                file,
                line,
                "targetless-arbiter",
                "multi-row ON CONFLICT DO NOTHING needs an explicit conflict target; targetless DO NOTHING can arbitrate different unique or exclusion constraints",
            ));
        }
        SqlConflictTarget::Columns {
            expressions,
            predicate,
        } => (
            catalog.resolve_columns(&insert.table, expressions, predicate.as_deref()),
            Some(expressions),
        ),
        SqlConflictTarget::Constraint(name) => {
            (catalog.resolve_constraint(&insert.table, name), None)
        }
    };
    let index = match arbiter {
        ResolvedArbiter::Exact(index) => index,
        ResolvedArbiter::Ambiguous => {
            return Some(finding(
                file,
                line,
                "ambiguous-arbiter",
                "ON CONFLICT infers multiple catalog arbiters with incompatible key orders; use ON CONSTRAINT or remove the incompatible index",
            ));
        }
        ResolvedArbiter::Unresolved => {
            return Some(finding(
                file,
                line,
                "unresolved-arbiter",
                "ON CONFLICT target does not resolve to one valid, ready btree unique index in schemaCatalogPath",
            ));
        }
    };
    if let Some(target) = target_is_ordered {
        if !target_matches_catalog(target, &index) {
            return Some(finding(
                file,
                line,
                "noncanonical-target",
                "ON CONFLICT target keys must use the resolved catalog index order",
            ));
        }
    }
    let Some(expected) = expected_source_order(&index, &insert.source) else {
        return Some(finding(
            file,
            line,
            "unresolved-source-order",
            "multi-row INSERT source must be a direct SELECT with explicit target columns so arbiter expressions can be mapped to ORDER BY",
        ));
    };
    let Some(actual) = insert.source.order.as_ref() else {
        return Some(finding(
            file,
            line,
            "missing-canonical-order",
            &format!(
                "multi-row ON CONFLICT must order its INSERT source by catalog arbiter {}",
                display_keys(&expected)
            ),
        ));
    };
    let actual = resolve_order_aliases(actual, &insert.source.order_aliases);
    if !order_prefix_matches(&actual, &expected, false) {
        return Some(finding(
            file,
            line,
            "noncanonical-order",
            &format!(
                "multi-row ON CONFLICT ORDER BY must begin with catalog arbiter {}",
                display_keys(&expected)
            ),
        ));
    }
    None
}

fn target_matches_catalog(target: &[String], index: &CanonicalIndex) -> bool {
    target.len() == index.keys.len()
        && target.iter().zip(&index.keys).all(|(actual, expected)| {
            crate::codebase::postgres::expression_matches(actual, &expected.expression, false)
        })
}

fn expected_source_order(
    index: &CanonicalIndex,
    source: &SqlInsertSourceShape,
) -> Option<Vec<CanonicalOrderKey>> {
    let projections = source.projections.as_ref()?;
    index
        .keys
        .iter()
        .map(|key| {
            Some(CanonicalOrderKey {
                expression: substitute_target_columns(&key.expression, projections)?,
                ascending: key.ascending,
                nulls_first: key.nulls_first,
            })
        })
        .collect()
}

fn display_keys(keys: &[CanonicalOrderKey]) -> String {
    keys.iter()
        .map(|key| {
            format!(
                "{} {} NULLS {}",
                key.expression,
                if key.ascending { "ASC" } else { "DESC" },
                if key.nulls_first { "FIRST" } else { "LAST" }
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn contains_insert_conflict(sql: &str) -> bool {
    let normalized = sql.to_ascii_lowercase();
    normalized.contains("insert") && normalized.contains("on conflict")
}

pub(super) fn contains_insert(sql: &str) -> bool {
    sql.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|token| token.eq_ignore_ascii_case("insert"))
}

fn resolve_order_aliases(
    order: &[CanonicalOrderKey],
    aliases: &std::collections::BTreeMap<String, String>,
) -> Vec<CanonicalOrderKey> {
    order
        .iter()
        .map(|key| CanonicalOrderKey {
            expression: aliases
                .get(&key.expression.to_ascii_lowercase())
                .cloned()
                .unwrap_or_else(|| key.expression.clone()),
            ascending: key.ascending,
            nulls_first: key.nulls_first,
        })
        .collect()
}

pub(super) fn finding(file: &str, line: usize, target: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: message.to_string(),
        import: None,
        target: Some(target.to_string()),
    }
}
