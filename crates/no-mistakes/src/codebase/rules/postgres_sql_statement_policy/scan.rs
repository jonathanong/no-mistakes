use super::{sql_rel, CompiledOptions, RuleFinding, RULE_ID};
use crate::codebase::postgres::collect_prepared_schema_facts;
use crate::codebase::ts_source::SourceStore;
use anyhow::Context;
use std::path::{Path, PathBuf};

pub(super) fn scan(
    root: &Path,
    opts: &CompiledOptions,
    files: &[PathBuf],
    sources: &SourceStore,
    facts: &crate::codebase::check_facts::CheckFactMap,
) -> anyhow::Result<Vec<RuleFinding>> {
    let schema_facts = collect_prepared_schema_facts(root, files, &opts.schema, facts)
        .context(format!("{RULE_ID} failed to collect PostgreSQL facts"))?;
    let mut findings = Vec::new();
    let mut dedup = crate::codebase::rules::VariantFindingDedup::default();
    for file in &schema_facts {
        let rel = sql_rel(root, &file.path);
        findings.extend(settings(&rel, &file.setting_uses, opts));
        for statement in &file.statement_kinds {
            if !opts.banned.contains(&statement.kind) {
                continue;
            }
            findings.push(RuleFinding {
                source_offset: None,
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
                if call.is_unanalyzable() {
                    findings.push(unanalyzable(&rel, call.line.max(1) as usize));
                }
            }
        }
        let source = crate::codebase::rules::read_source(sources, path);
        for file in facts.postgres_statements(path, Some(&opts.embedded))? {
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
                            source_offset: None,
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
    crate::codebase::rules::sort_postgres_findings(&mut findings);
    Ok(findings)
}

fn unanalyzable(file: &str, line: usize) -> RuleFinding {
    RuleFinding {
        source_offset: None,
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
            source_offset: None,
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
