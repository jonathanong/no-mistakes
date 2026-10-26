use super::{CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{
    statements::SqlFactSite, SqlBranchJudgment, SqlStatementFileFacts,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    options: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let prepared = facts.prepared();
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    for path in files {
        let rel = relative_slash_path(root, path);
        let embedded = prepared.embedded_sql(path, &options.embedded)?;
        let statements = prepared.postgres_statements(path, Some(&options.embedded))?;
        if facts.wants_sql_verdicts() {
            crate::codebase::postgres::record_sql_policy_verdicts(
                crate::codebase::postgres::SqlPolicyVerdictContext {
                    facts,
                    embedded,
                    statements,
                },
                RULE_ID,
                &rel,
                &options.embedded,
                &["test-state-mutation".to_string()],
                options.advise_unanalyzable,
                |_, _, statement| {
                    if violates(statement) {
                        SqlBranchJudgment::violation(true)
                    } else {
                        SqlBranchJudgment::clear()
                    }
                },
            );
        }
        if options.fail_unanalyzable {
            let mut projected = statements.iter();
            for call in &embedded.calls {
                let mut known_mutation = false;
                for branch in call.statement_calls() {
                    if branch.sql_text.is_some() {
                        known_mutation |= projected.next().is_some_and(violates);
                    }
                }
                if call.is_unanalyzable() && !known_mutation {
                    findings.push(unknown(&rel, call.line as usize));
                }
            }
        }
        let source = super::super::read_source(sources, path);
        for statement in statements {
            if options.fail_unanalyzable && !violates(statement) {
                for (site, line) in statement.parse_failure_sites() {
                    dedup.push(
                        &mut findings,
                        statement,
                        site,
                        source.as_deref(),
                        unknown(&rel, line),
                    );
                }
            }
            for (index, kind) in statement.statement_kinds.iter().enumerate() {
                if banned_kind(&kind.kind) {
                    dedup.push(
                        &mut findings,
                        statement,
                        SqlFactSite::StatementKind(index),
                        source.as_deref(),
                        finding(&rel, kind.line, &kind.kind),
                    );
                }
            }
            for (index, setting) in statement.setting_uses.iter().enumerate() {
                if setting.name == "session_replication_role" {
                    dedup.push(
                        &mut findings,
                        statement,
                        SqlFactSite::Setting(index),
                        source.as_deref(),
                        finding(&rel, setting.line, "session_replication_role"),
                    );
                }
            }
            for (index, call) in statement.function_calls.iter().enumerate() {
                if is_pg_sleep(&call.name_parts) {
                    dedup.push(
                        &mut findings,
                        statement,
                        SqlFactSite::Function(index),
                        source.as_deref(),
                        finding(&rel, call.line, "pg_sleep"),
                    );
                }
            }
        }
    }
    findings.sort();
    Ok(findings)
}

fn violates(facts: &SqlStatementFileFacts) -> bool {
    facts
        .statement_kinds
        .iter()
        .any(|item| banned_kind(&item.kind))
        || facts
            .setting_uses
            .iter()
            .any(|item| item.name == "session_replication_role")
        || facts
            .function_calls
            .iter()
            .any(|item| is_pg_sleep(&item.name_parts))
}

pub(super) fn banned_kind(kind: &str) -> bool {
    if kind == "TRUNCATE" || kind == "ALTER SYSTEM" {
        return true;
    }
    let Some((verb, object)) = kind.split_once(' ') else {
        return false;
    };
    matches!(verb, "CREATE" | "ALTER" | "DROP")
        && matches!(
            object,
            "DATABASE"
                | "SCHEMA"
                | "TABLE"
                | "VIEW"
                | "MATERIALIZED VIEW"
                | "TRIGGER"
                | "FUNCTION"
                | "PROCEDURE"
                | "INDEX"
                | "TYPE"
        )
}

pub(super) fn is_pg_sleep(parts: &[String]) -> bool {
    matches!(parts, [name] if name == "pg_sleep")
        || matches!(parts, [schema, name] if schema == "pg_catalog" && name == "pg_sleep")
}

fn finding(file: &str, line: usize, operation: &str) -> RuleFinding {
    let line = line.max(1);
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message: format!(
            "{file}:{line}: executed test SQL uses {operation}; keep tests isolated: run migrations before the suite or inject a throwing executor for failure tests"
        ),
        import: None,
        target: Some(operation.into()),
    }
}

fn unknown(file: &str, line: usize) -> RuleFinding {
    let line = line.max(1);
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message: format!(
            "{file}:{line}: executed test SQL is not statically recoverable, so database state mutation cannot be ruled out; pass a SQL literal or trusted tagged template"
        ),
        import: None,
        target: Some("unanalyzable-sql".into()),
    }
}
