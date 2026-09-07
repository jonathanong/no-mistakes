use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{collect_postgres_facts, EmbeddedSqlKind};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
    if opts.relations.is_empty() {
        return Ok(Vec::new());
    }
    let facts = collect_postgres_facts(
        root,
        sources,
        files,
        &CheckFactPlan {
            postgres_dml: true,
            embedded_sql: true,
            ..CheckFactPlan::default()
        },
        &opts.schema,
        &opts.embedded,
    )
    .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &facts.embedded {
        let rel = relative_slash_path(root, &file.path);
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for required predicates",
                    Some("unanalyzable"),
                ));
            }
        }
    }
    for file in &facts.statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(finding(
                &rel,
                1,
                "SQL could not be analyzed for required predicates",
                Some("unanalyzable"),
            ));
            continue;
        }
        for select in &file.selects {
            findings.extend(select_findings(&rel, select, opts));
        }
    }
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

fn select_findings(
    file: &str,
    select: &crate::codebase::postgres::SqlSelectFact,
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
            if contains_predicate(&select.predicate_sql, required) {
                continue;
            }
            findings.push(finding(
                file,
                select.line.max(1),
                &format!(
                    "queries against {} must include `{}`",
                    relation.table, required
                ),
                Some(relation.table.as_str()),
            ));
        }
    }
    findings
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

fn finding(file: &str, line: usize, message: &str, target: Option<&str>) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: target.map(ToOwned::to_owned),
    }
}
