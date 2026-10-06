use super::{iteration, BannedShapes, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactMap;
use crate::codebase::postgres::{postgres_sql_paths, EmbeddedSqlKind};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    facts: &CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let file = facts
            .embedded_sql(path, &opts.embedded)
            .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
            }
        }
        for fragment in &file.fragments {
            if fragment.sql_text.is_none() && opts.fail_unanalyzable {
                findings.push(finding(
                    &rel,
                    fragment.line.max(1) as usize,
                    "builder SQL is not statically recoverable for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
            }
        }
        for fragment in facts.postgres_fragments(path, &opts.embedded)? {
            let statements = &fragment.statements;
            findings.extend(super::functions::findings(
                &rel,
                statements,
                &opts.shapes,
                &opts.banned_functions,
                |line| fragment.line.saturating_add(line as u32).saturating_sub(1) as usize,
            ));
            if opts.fail_unanalyzable && statements.parse_failed {
                findings.push(finding(
                    &rel,
                    fragment.line.max(1) as usize,
                    "builder SQL is not statically recoverable for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
                continue;
            }
            let line_at =
                |line: usize| fragment.line.saturating_add(line as u32).saturating_sub(1) as usize;
            for select in &statements.selects {
                findings.extend(select_findings(&rel, select, &opts.shapes, line_at));
            }
            findings.extend(iteration::findings(
                &rel,
                statements,
                &opts.shapes,
                &opts.iteration,
                line_at,
            ));
        }
    }
    let sql_paths = postgres_sql_paths(root, files, &opts.schema)?;
    let projections = sql_paths.iter().map(|path| (path, None)).chain(
        files
            .iter()
            .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
            .map(|path| (path, Some(&opts.embedded))),
    );
    for (path, profile) in projections {
        for file in facts.postgres_statements(path, profile)? {
            let rel = relative_slash_path(root, &file.path);
            findings.extend(super::functions::findings(
                &rel,
                file,
                &opts.shapes,
                &opts.banned_functions,
                |line| line.max(1),
            ));
            if opts.fail_unanalyzable && file.parse_failed {
                findings.push(finding(
                    &rel,
                    file.origin_line.max(1),
                    "SQL could not be analyzed for shape policy",
                    opts.shapes.unanalyzable_target(),
                ));
                continue;
            }
            for select in &file.selects {
                findings.extend(select_findings(&rel, select, &opts.shapes, |line| {
                    line.max(1)
                }));
            }
            findings.extend(iteration::findings(
                &rel,
                file,
                &opts.shapes,
                &opts.iteration,
                |line| line.max(1),
            ));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    findings.dedup_by(|left, right| {
        left.rule == right.rule
            && left.file == right.file
            && left.line == right.line
            && left.message == right.message
            && left.target == right.target
    });
    Ok(findings)
}

fn select_findings(
    file: &str,
    select: &crate::codebase::postgres::SqlSelectFact,
    shapes: &BannedShapes,
    line_at: impl Fn(usize) -> usize,
) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    if shapes.correlated_exists_set_operation {
        for exists in &select.exists_set_operations {
            if exists.correlated {
                findings.push(finding(
                    file,
                    line_at(exists.line),
                    "do not wrap a set operation in a correlated EXISTS",
                    "correlated-exists-set-operation",
                ));
            }
        }
    }
    if shapes.not_in_subquery {
        for line in &select.not_in_subqueries {
            findings.push(finding(
                file,
                line_at(*line),
                "NOT IN (SELECT …) returns no rows when the subquery yields a NULL and cannot become an anti-join; use NOT EXISTS (SELECT 1 FROM … WHERE …)",
                "not-in-subquery",
            ));
        }
    }
    if shapes.count_for_existence {
        for count in &select.count_existence_checks {
            findings.push(finding(
                file,
                line_at(count.line),
                if count.negated { "COUNT(*) compared with 0/1 counts every matching row to test absence; use NOT EXISTS (SELECT 1 FROM … WHERE …)" } else { "COUNT(*) compared with 0/1 counts every matching row to test existence; use EXISTS (SELECT 1 FROM … WHERE …)" },
                "count-for-existence",
            ));
        }
    }
    findings
}

pub(super) fn finding(file: &str, line: usize, message: &str, target: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.to_string()),
    }
}
