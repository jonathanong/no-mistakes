use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::collect_prepared_schema_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    _sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let schema_facts = collect_prepared_schema_facts(root, files, &opts.schema, facts)
        .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    for file in &schema_facts {
        let rel = sql_rel(root, &file.path);
        for statement in &file.statement_kinds {
            if !opts.banned.contains(&statement.kind) {
                continue;
            }
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: statement.line.max(1),
                message: format!(
                    "{rel}:{}: SQL files matching this rule must not use {}",
                    statement.line.max(1),
                    statement.kind
                ),
                import: None,
                target: Some(statement.kind.clone()),
            });
        }
    }
    if !opts.embedded.selects_executors() {
        return Ok(findings);
    }
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let embedded = facts.embedded_sql(path, &opts.embedded)?;
        let rel = sql_rel(root, path);
        if opts.fail_unanalyzable {
            for call in &embedded.calls {
                if call.kind == crate::codebase::postgres::EmbeddedSqlKind::Dynamic {
                    findings.push(unanalyzable(&rel, call.line.max(1) as usize));
                }
            }
        }
        for file in facts.postgres_statements(path, Some(&opts.embedded))? {
            if file.parse_failed && opts.fail_unanalyzable {
                findings.push(unanalyzable(&rel, file.origin_line.max(1)));
                continue;
            }
            for statement in &file.statement_kinds {
                if opts.banned.contains(&statement.kind) {
                    findings.push(RuleFinding {
                        rule: RULE_ID.into(),
                        file: rel.clone(),
                        line: statement.line.max(1),
                        message: format!(
                            "{rel}:{}: executed SQL must not use {}",
                            statement.line.max(1),
                            statement.kind
                        ),
                        import: None,
                        target: Some(statement.kind.clone()),
                    });
                }
            }
        }
    }
    Ok(findings)
}

fn unanalyzable(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message: format!(
            "{file}:{line}: executed SQL is not statically recoverable for statement policy"
        ),
        import: None,
        target: Some("unanalyzable-sql".into()),
    }
}
