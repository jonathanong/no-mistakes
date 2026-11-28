use super::{CompiledOptions, RULE_ID};
use crate::codebase::postgres::statements::SqlStarProjectionFact;
use crate::codebase::postgres::SchemaCatalog;
use no_mistakes_rule_kernel::finding::RuleFinding;

pub(super) fn sql_finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}

pub(super) fn star_findings(
    file: &str,
    stars: &[SqlStarProjectionFact],
    kind: &str,
    verb: &str,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    for star in stars {
        if let Some(function) = &star.within_function {
            if opts
                .whole_row_functions
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(function))
            {
                continue;
            }
        }
        let Some(text) = message(kind, verb, star, opts, catalog) else {
            continue;
        };
        findings.push(sql_finding(file, star.line.max(1), &text, &star.relation));
    }
    findings
}

pub(super) fn message(
    kind: &str,
    verb: &str,
    star: &SqlStarProjectionFact,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> Option<String> {
    relation_message(kind, verb, &star.relation, opts, catalog)
}

pub(super) fn relation_message(
    kind: &str,
    verb: &str,
    relation: &str,
    opts: &CompiledOptions,
    catalog: &SchemaCatalog,
) -> Option<String> {
    let listed = opts
        .relations
        .iter()
        .any(|configured| configured.eq_ignore_ascii_case(relation));
    if listed {
        return Some(format!(
            "{kind} {verb} all columns of {relation}, which is configured in relations; list the columns the caller uses",
        ));
    }
    let info = catalog.relation(relation)?;
    if opts.max_columns == 0 {
        return Some(format!(
            "{kind} {verb} every column of {relation}, including columns added later; list the columns the caller uses",
        ));
    }
    let width = info.columns.len();
    if width <= opts.max_columns as usize {
        return None;
    }
    Some(format!(
        "{kind} {verb} all {width} columns of {relation} (limit {limit}); list the columns the caller uses",
        limit = opts.max_columns
    ))
}
