use super::{clauses::clause_finding, sql_rel, CompiledOptions, RULE_ID};
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::{collect_prepared_schema_facts, statements::SqlFactSite};
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

mod verdicts;

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &dyn SqlFactContext,
) -> anyhow::Result<crate::codebase::rules::PostgresFindings> {
    let prepared = facts.prepared();
    let schema_facts = collect_prepared_schema_facts(root, files, &opts.schema, prepared)
        .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = crate::codebase::rules::PostgresFindings::default();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::for_root(root, sources);
    for file in &schema_facts {
        let rel = sql_rel(root, &file.path);
        findings.extend(settings(&rel, &file.setting_uses, opts));
        findings.extend(
            file.clause_uses
                .iter()
                .filter_map(|clause| clause_finding(&rel, clause, opts)),
        );
        for statement in &file.statement_kinds {
            if !opts.banned.contains(&statement.kind) {
                continue;
            }
            findings.push(RuleFinding {
                causes: Vec::new(),
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
    let checks = if facts.wants_sql_verdicts() {
        verdicts::checks(opts)
    } else {
        Vec::new()
    };
    for path in files
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
    {
        let embedded = prepared.embedded_sql(path, &opts.embedded)?;
        let rel = sql_rel(root, path);
        let statements = prepared.postgres_statements(path, Some(&opts.embedded))?;
        crate::codebase::postgres::record_sql_policy_verdicts(
            crate::codebase::postgres::SqlPolicyVerdictContext {
                facts,
                embedded,
                statements,
            },
            RULE_ID,
            &rel,
            &opts.embedded,
            &checks,
            opts.advise_unanalyzable,
            |check, _, statement| verdicts::judge(check, statement),
        );
        if opts.fail_unanalyzable {
            for call in &embedded.calls {
                if call.is_unanalyzable() {
                    findings.push(unanalyzable(&rel, call.line.max(1) as usize));
                }
            }
        }
        let source = no_mistakes_rule_kernel::source_access::read_language_source(sources, path);
        for file in statements {
            if file.parse_failed && opts.fail_unanalyzable {
                for (site, line) in file.parse_failure_sites() {
                    dedup.push(
                        &mut findings,
                        file,
                        site,
                        source.as_deref(),
                        unanalyzable(&rel, line),
                    );
                }
            }
            for (index, setting) in file.setting_uses.iter().enumerate() {
                for finding in settings(&rel, std::slice::from_ref(setting), opts) {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::Setting(index),
                        source.as_deref(),
                        finding,
                    );
                }
            }
            for (index, clause) in file.clause_uses.iter().enumerate() {
                if let Some(finding) = clause_finding(&rel, clause, opts) {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::Clause(index),
                        source.as_deref(),
                        finding,
                    );
                }
            }
            for (index, statement) in file.statement_kinds.iter().enumerate() {
                if opts.banned.contains(&statement.kind) {
                    dedup.push(
                        &mut findings,
                        file,
                        SqlFactSite::StatementKind(index),
                        source.as_deref(),
                        RuleFinding {
                            causes: Vec::new(),
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
                        },
                    );
                }
            }
        }
    }
    findings.sort();
    Ok(findings)
}

fn unanalyzable(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
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

fn settings(
    file: &str,
    uses: &[crate::codebase::postgres::SqlSettingUse],
    opts: &CompiledOptions,
) -> Vec<RuleFinding> {
    uses.iter()
        .filter(|setting| opts.settings.contains(&setting.name))
        .map(|setting| RuleFinding {
            causes: Vec::new(),
            rule: RULE_ID.into(),
            file: file.into(),
            line: setting.line.max(1),
            message: format!(
                "{file}:{}: SQL matching this rule must not change PostgreSQL setting `{}`",
                setting.line.max(1),
                setting.name
            ),
            import: None,
            target: Some(format!("setting:{}", setting.name)),
        })
        .collect()
}
