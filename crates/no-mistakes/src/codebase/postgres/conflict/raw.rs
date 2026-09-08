use super::SqlConflictTarget;
use anyhow::Result;
use lex::{
    find_keyword, matching_parenthesis, skip_whitespace, split_top_level, starts_keyword,
    take_identifier,
};

mod lex;

#[derive(Debug, Clone)]
pub(super) struct RawConflict {
    pub(super) target: SqlConflictTarget,
    replacements: Vec<(usize, usize, String)>,
}

pub(super) fn raw_conflicts(sql: &str) -> Result<Vec<RawConflict>> {
    let mut conflicts = Vec::new();
    let mut start = 0usize;
    while let Some(on) = find_keyword(sql, "on", start) {
        let Some(conflict) = find_keyword(sql, "conflict", on + 2) else {
            start = on + 2;
            continue;
        };
        if !sql[on + 2..conflict].trim().is_empty() {
            start = on + 2;
            continue;
        }
        let after = skip_whitespace(sql, conflict + "conflict".len());
        let (target, replacements, next) = if starts_keyword(sql, after, "on")
            && starts_keyword(sql, skip_whitespace(sql, after + 2), "constraint")
        {
            let constraint_start = skip_whitespace(sql, after + "on".len());
            let name_start = skip_whitespace(sql, constraint_start + "constraint".len());
            let (name, end) = take_identifier(sql, name_start)?;
            (SqlConflictTarget::Constraint(name), Vec::new(), end)
        } else if sql.as_bytes().get(after) == Some(&b'(') {
            let close = matching_parenthesis(sql, after)?;
            let expressions = split_top_level(&sql[after + 1..close]);
            let do_start = find_keyword(sql, "do", close + 1)
                .ok_or_else(|| anyhow::anyhow!("ON CONFLICT has no DO action"))?;
            let predicate_start =
                find_keyword(sql, "where", close + 1).filter(|where_start| *where_start < do_start);
            let predicate = predicate_start.map(|where_start| {
                sql[where_start + "where".len()..do_start]
                    .trim()
                    .to_string()
            });
            let mut replacements = vec![(after + 1, close, "nm_conflict_key".to_string())];
            if let Some(where_start) = predicate_start {
                replacements.push((where_start, do_start, " ".to_string()));
            }
            (
                SqlConflictTarget::Columns {
                    expressions,
                    predicate,
                },
                replacements,
                close + 1,
            )
        } else {
            (SqlConflictTarget::Targetless, Vec::new(), after)
        };
        conflicts.push(RawConflict {
            target,
            replacements,
        });
        start = next;
    }
    Ok(conflicts)
}

pub(super) fn sanitize(sql: &str, conflicts: &[RawConflict]) -> String {
    let mut replacements = conflicts
        .iter()
        .flat_map(|conflict| conflict.replacements.iter().cloned())
        .collect::<Vec<_>>();
    replacements.sort_by_key(|replacement| std::cmp::Reverse(replacement.0));
    let mut sanitized = sql.to_string();
    for (start, end, replacement) in replacements {
        sanitized.replace_range(start..end, &replacement);
    }
    sanitized
}
