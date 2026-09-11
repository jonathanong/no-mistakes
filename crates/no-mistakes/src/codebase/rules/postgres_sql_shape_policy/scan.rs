use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::CheckFactPlan;
use crate::codebase::postgres::{
    collect_postgres_facts, statements::extract_sql_statement_facts, EmbeddedSqlKind,
};
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Context;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> anyhow::Result<Vec<RuleFinding>> {
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
        let executed_sql: HashSet<&str> = file
            .calls
            .iter()
            .filter(|call| call.kind != EmbeddedSqlKind::Dynamic)
            .filter_map(|call| call.sql_text.as_deref())
            .collect();
        for call in &file.calls {
            if opts.fail_unanalyzable && call.kind == EmbeddedSqlKind::Dynamic {
                findings.push(finding(
                    &rel,
                    call.line.max(1) as usize,
                    "executed SQL is not statically recoverable for shape policy",
                ));
            }
        }
        if !opts.ban_exists_set_op {
            continue;
        }
        for fragment in &file.fragments {
            let Some(sql_text) = fragment.sql_text.as_deref() else {
                if opts.fail_unanalyzable {
                    findings.push(finding(
                        &rel,
                        fragment.line.max(1) as usize,
                        "builder SQL is not statically recoverable for shape policy",
                    ));
                }
                continue;
            };
            if executed_sql.contains(sql_text) {
                continue;
            }
            let statements = fragment_statement_facts(sql_text);
            if opts.fail_unanalyzable && statements.parse_failed {
                findings.push(finding(
                    &rel,
                    fragment.line.max(1) as usize,
                    "builder SQL is not statically recoverable for shape policy",
                ));
                continue;
            }
            for select in &statements.selects {
                for exists in &select.exists_set_operations {
                    if exists.correlated {
                        findings.push(finding(
                            &rel,
                            fragment
                                .line
                                .saturating_add(exists.line as u32)
                                .saturating_sub(1) as usize,
                            "do not wrap a set operation in a correlated EXISTS",
                        ));
                    }
                }
            }
        }
    }
    for file in &facts.statements {
        let rel = relative_slash_path(root, &file.path);
        if opts.fail_unanalyzable && file.parse_failed {
            findings.push(finding(
                &rel,
                1,
                "SQL could not be analyzed for shape policy",
            ));
            continue;
        }
        if !opts.ban_exists_set_op {
            continue;
        }
        for select in &file.selects {
            for exists in &select.exists_set_operations {
                if exists.correlated {
                    findings.push(finding(
                        &rel,
                        exists.line.max(1),
                        "do not wrap a set operation in a correlated EXISTS",
                    ));
                }
            }
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

fn fragment_statement_facts(sql: &str) -> crate::codebase::postgres::SqlStatementFileFacts {
    let direct = extract_sql_statement_facts(sql);
    if !direct.selects.is_empty() {
        return direct;
    }
    // Builder fragments commonly begin with `EXISTS` or `AND EXISTS`, which
    // are valid predicate fragments but not top-level SQL statements.
    let prefix = sql.trim_start();
    let wrapper = if prefix.starts_with("AND ") || prefix.starts_with("OR ") {
        format!("SELECT 1 WHERE true {sql}")
    } else {
        format!("SELECT 1 WHERE {sql}")
    };
    extract_sql_statement_facts(&wrapper)
}

fn finding(file: &str, line: usize, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some("correlated-exists-set-operation".to_string()),
    }
}
