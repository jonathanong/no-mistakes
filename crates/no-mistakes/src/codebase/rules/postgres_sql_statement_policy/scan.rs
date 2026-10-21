use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::check_facts::SqlFactContext;
use crate::codebase::postgres::collect_prepared_schema_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

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
        let embedded = prepared.embedded_sql(path, &opts.embedded)?;
        let rel = sql_rel(root, path);
        let statements = prepared.postgres_statements(path, Some(&opts.embedded))?;
        if facts.wants_sql_verdicts() {
            let mut checks = opts
                .banned
                .iter()
                .map(|kind| format!("statement:{kind}"))
                .chain(opts.settings.iter().map(|name| format!("setting:{name}")))
                .collect::<Vec<_>>();
            checks.sort();
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
                |check, _, statement| {
                    let violates = check.strip_prefix("statement:").map_or_else(
                        || {
                            check.strip_prefix("setting:").is_some_and(|name| {
                                statement
                                    .setting_uses
                                    .iter()
                                    .any(|setting| setting.name == name)
                            })
                        },
                        |kind| {
                            statement
                                .statement_kinds
                                .iter()
                                .any(|item| item.kind == kind)
                        },
                    );
                    if violates {
                        crate::codebase::postgres::SqlBranchJudgment::violation(true)
                    } else {
                        crate::codebase::postgres::SqlBranchJudgment::clear()
                    }
                },
            );
        }
        if opts.fail_unanalyzable {
            for call in &embedded.calls {
                if call.is_unanalyzable() {
                    findings.push(unanalyzable(&rel, call.line.max(1) as usize));
                }
            }
        }
        let source = crate::codebase::rules::read_source(sources, path);
        for file in statements {
            if file.parse_failed && opts.fail_unanalyzable {
                dedup.push(
                    &mut findings,
                    file,
                    crate::codebase::postgres::statements::SqlFactSite::Origin,
                    source.as_deref(),
                    unanalyzable(&rel, file.origin_line.max(1)),
                );
            }
            for (index, setting) in file.setting_uses.iter().enumerate() {
                for finding in settings(&rel, std::slice::from_ref(setting), opts) {
                    dedup.push(
                        &mut findings,
                        file,
                        crate::codebase::postgres::statements::SqlFactSite::Setting(index),
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
                        crate::codebase::postgres::statements::SqlFactSite::StatementKind(index),
                        source.as_deref(),
                        RuleFinding {
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
