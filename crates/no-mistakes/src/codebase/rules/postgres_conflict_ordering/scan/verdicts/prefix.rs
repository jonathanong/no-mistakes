use super::analysis;
use crate::codebase::postgres::{
    analyze_conflict_inserts, EmbeddedSqlCall, SchemaCatalog, SqlBranchJudgment,
};
use crate::codebase::rules::RuleFinding;

/// A trailing interpolation cannot insert a conflict target into a clause that
/// already says `ON CONFLICT DO NOTHING`. Order findings still depend on that tail.
pub(super) fn dynamic_prefix_judgment(
    call: &EmbeddedSqlCall,
    catalog: &SchemaCatalog,
) -> SqlBranchJudgment {
    let Some(prefix) = known_sql_prefix(call) else {
        return SqlBranchJudgment::abstain("opaque-sql-tail");
    };
    if !analysis::contains_insert_conflict(prefix) {
        return SqlBranchJudgment::abstain("opaque-sql-tail");
    }
    let Ok(inserts) = analyze_conflict_inserts(prefix) else {
        return SqlBranchJudgment::abstain("opaque-sql-tail");
    };
    let findings = analysis::findings_for_inserts("", call.line as usize, &inserts, catalog);
    match targetless_survives(&findings) {
        Some(true) => SqlBranchJudgment::violation(true),
        _ => SqlBranchJudgment::abstain("opaque-sql-tail"),
    }
}

/// `Some(true)` when every proven finding is `targetless-arbiter`.
/// Catalog uncertainty is not a proven finding. `None` means there is none.
pub(super) fn targetless_survives(findings: &[(usize, RuleFinding)]) -> Option<bool> {
    let mut survives = None;
    for (_, finding) in findings {
        let target = finding.target.as_deref().unwrap_or_default();
        if matches!(
            target,
            "unresolved-arbiter" | "ambiguous-arbiter" | "unresolved-source-order"
        ) {
            continue;
        }
        survives = Some(survives.unwrap_or(true) && target == "targetless-arbiter");
    }
    survives
}

fn known_sql_prefix(call: &EmbeddedSqlCall) -> Option<&str> {
    let sql = call.sql_text.as_deref()?;
    let end = earliest_placeholder(sql, &call.recovered_placeholder_positions);
    let prefix = sql[..end].trim();
    (!prefix.is_empty()).then_some(prefix)
}

fn earliest_placeholder(sql: &str, positions: &[(u32, u32)]) -> usize {
    positions
        .iter()
        .filter_map(|(line, column)| byte_offset(sql, *line, *column))
        .min()
        .or_else(|| sql.find("sql_placeholder_"))
        .unwrap_or(sql.len())
}

fn byte_offset(sql: &str, line: u32, column: u32) -> Option<usize> {
    let mut current_line = 1u32;
    let mut current_column = 1u32;
    for (index, character) in sql.char_indices() {
        if current_line == line && current_column == column {
            return Some(index);
        }
        if character == '\n' {
            current_line += 1;
            current_column = 1;
        } else {
            current_column += 1;
        }
    }
    (current_line == line && current_column == column).then_some(sql.len())
}
