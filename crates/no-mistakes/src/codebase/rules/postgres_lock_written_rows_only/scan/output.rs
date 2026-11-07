use super::*;
use crate::codebase::postgres::{SqlObligationVerdict, SqlVerdictStatus};
pub(super) struct Site<'a> {
    pub file: &'a str,
    pub source: &'a str,
    pub call: &'a EmbeddedSqlCall,
}
pub(super) struct Output<'a> {
    options: &'a CompiledOptions,
    facts: &'a dyn SqlFactContext,
    findings: crate::codebase::rules::PostgresFindings,
}
impl<'a> Output<'a> {
    pub fn new(options: &'a CompiledOptions, facts: &'a dyn SqlFactContext) -> Self {
        Self {
            options,
            facts,
            findings: Default::default(),
        }
    }
    pub fn write_set(&mut self, site: &Site<'_>, relation: &str) -> bool {
        self.options
            .write_sets_audit
            .as_ref()
            .expect("prepared audit")
            .usage
            .suppress(self.options.write_sets.iter().map(|entry| {
                entry.file == site.file
                    && Some(entry.transaction_line) == site.call.transaction_line
                    && entry
                        .relations
                        .iter()
                        .any(|name| relation_matches(name, relation))
            }))
    }
    pub fn unresolved(&mut self, site: &Site<'_>, message: &str) {
        self.options.incomplete_files.insert(site.file.into());
        self.push(
            site,
            "unanalyzable",
            None,
            None,
            message,
            self.options.unanalyzable,
        );
    }
    pub fn emit(
        &mut self,
        site: &Site<'_>,
        check: &str,
        relation: Option<&str>,
        key: Option<&str>,
        message: &str,
    ) {
        self.push(
            site,
            check,
            relation,
            key,
            message,
            self.options.checks.get(check).copied().unwrap_or_default(),
        );
    }
    fn push(
        &mut self,
        site: &Site<'_>,
        check: &str,
        relation: Option<&str>,
        key: Option<&str>,
        message: &str,
        mode: Mode,
    ) {
        if mode == Mode::Ignore {
            return;
        }
        let allowed =
            self.options
                .allow_audit
                .as_ref()
                .expect("prepared audit")
                .usage
                .suppress(
                    self.options.allow.iter().map(|entry| {
                        entry.matches(site.file, site.call.line as usize, relation, key)
                    }),
                );
        if allowed {
            return;
        }
        if mode == Mode::Fail {
            self.findings.push(finding(
                site.file,
                site.call.line as usize,
                check,
                &format!("{check}: {message}"),
            ));
        } else {
            let start = site
                .source
                .split_inclusive('\n')
                .take(site.call.line.saturating_sub(1) as usize)
                .map(str::len)
                .sum::<usize>() as u32;
            self.facts.record_sql_advisory(SqlObligationVerdict {
                profile: self.options.embedded.clone(),
                rule: RULE_ID.into(),
                application: None,
                subcheck: check.into(),
                file: site.file.into(),
                span: (start, start),
                status: if check == "unanalyzable" {
                    SqlVerdictStatus::Abstain
                } else {
                    SqlVerdictStatus::Violation
                },
                advisory: true,
                evidence: None,
                known_prefix: None,
                helper_lines: vec![],
                residual_reasons: vec![message.to_owned()],
            });
        }
    }
    pub fn into_findings(self) -> crate::codebase::rules::PostgresFindings {
        self.findings
    }
    pub fn extend(&mut self, findings: crate::codebase::rules::PostgresFindings) {
        self.findings.extend(findings);
    }
    fn complete(&self, file: Option<&str>) -> bool {
        file.map_or_else(
            || !self.options.checked_files.is_empty() && self.options.incomplete_files.is_empty(),
            |file| {
                self.options.checked_files.contains(file)
                    && !self.options.incomplete_files.contains(file)
            },
        )
    }
    pub fn finish(mut self) -> crate::codebase::rules::PostgresFindings {
        self.findings.extend(
            self.options
                .allow_audit
                .as_ref()
                .expect("prepared audit")
                .finish(
                    |index| self.complete(self.options.allow[index].file.as_deref()),
                    |index| format!("allow[{}]", index + 1),
                ),
        );
        self.findings.extend(
            self.options
                .write_sets_audit
                .as_ref()
                .expect("prepared audit")
                .finish(
                    |index| self.complete(Some(&self.options.write_sets[index].file)),
                    |index| {
                        format!(
                            "{}:{}",
                            self.options.write_sets[index].file,
                            self.options.write_sets[index].transaction_line
                        )
                    },
                ),
        );
        self.findings
    }
}
fn finding(file: &str, line: usize, target: &str, message: &str) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.into(),
        file: file.into(),
        line,
        message: format!("{file}:{line}: {message}"),
        import: None,
        target: Some(target.into()),
    }
}
