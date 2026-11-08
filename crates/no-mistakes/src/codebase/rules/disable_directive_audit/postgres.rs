use super::*;
use crate::codebase::check_facts::{CheckFactMap, SqlFactContext};
use crate::codebase::postgres::EmbeddedSqlKind;
mod locations;
mod low;
use crate::codebase::ts_source::{disable_directives, has_disable_comment};
use locations::{collect, SqlText};

struct AuditScope<'a> {
    root: &'a Path,
    rule: &'a str,
    selected: &'a BTreeSet<PathBuf>,
    sources: &'a SourceStore,
}

impl PreparedDisableDirectives {
    #[doc(hidden)]
    pub fn findings_with_postgres(
        &self,
        config: &NoMistakesConfig,
        files: &[PathBuf],
        sources: &SourceStore,
        facts: &CheckFactMap,
        consumed: &[ConsumedDisableDirective],
    ) -> Result<Vec<RuleFinding>> {
        let mut findings = self.findings_from_consumed(consumed);
        if config.report_unused_disable_directives == DisableDirectivePolicy::Ignore {
            return Ok(findings);
        }
        let mut comments = Vec::new();
        for rule_id in crate::codebase::postgres::PREPARED_EMBEDDED_SQL_RULE_IDS {
            for (ordinal, rule) in config.rule_applications(rule_id).into_iter().enumerate() {
                let scoped = NoMistakesConfig {
                    rules: vec![rule.clone()],
                    ..config.clone()
                };
                let profiles = crate::codebase::postgres::configured_embedded_sql_options(
                    &scoped,
                    &[*rule_id],
                )?;
                let context = facts.for_sql_application_with_advice(ordinal, None);
                let selected = super::super::selected_rule_files(&self.root, config, rule, files)?;
                let selected: BTreeSet<_> = selected.into_iter().collect();
                let scope = AuditScope {
                    root: &self.root,
                    rule: rule_id,
                    selected: &selected,
                    sources,
                };
                let extra: Vec<_> = selected
                    .par_iter()
                    .flat_map_iter(|path| {
                        let mut comments = Vec::new();
                        if let Some(file) = facts.ts.get(path) {
                            for key in file
                                .sql_identifier_syntax
                                .application_keys()
                                .iter()
                                .filter(|key| key.rule == *rule_id && key.index == ordinal)
                            {
                                low::collect(&mut comments, &scope, facts, path, key);
                            }
                        }
                        for profile in &profiles {
                            let Ok(embedded) = context.embedded_sql(path, profile) else {
                                continue;
                            };
                            for call in &embedded.calls {
                                if !call.sql_text.as_deref().is_some_and(has_directive)
                                    && !call
                                        .variants
                                        .iter()
                                        .any(|variant| has_directive(&variant.sql_text))
                                {
                                    continue;
                                }
                                for call in call.statement_calls() {
                                    collect(
                                        &mut comments,
                                        &scope,
                                        path,
                                        SqlText {
                                            sql: call.sql_text.as_deref().unwrap_or_default(),
                                            positions: &call.sql_source_positions,
                                            base: match call.kind {
                                                EmbeddedSqlKind::Inline => call.line,
                                                _ => call.declaration_line.unwrap_or(call.line),
                                            },
                                        },
                                    );
                                }
                            }
                        }
                        comments
                    })
                    .collect();
                comments.extend(extra);
            }
        }
        findings.extend(
            Self {
                root: self.root.clone(),
                comments,
            }
            .findings_from_consumed(consumed),
        );
        super::super::sort_findings(&mut findings);
        Ok(findings)
    }
}

fn has_directive(sql: &str) -> bool {
    sql.contains("no-mistakes-disable-")
}

#[cfg(test)]
mod tests;
