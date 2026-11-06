use super::{CompiledOptions, Options};
use crate::codebase::postgres::{EmbeddedSqlFileFacts, SqlStatementFileFacts};
use crate::codebase::rules::allow_audit::AllowAudit;
use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

pub(super) struct SweepAudit {
    pub(super) audit: AllowAudit,
    selectors: Vec<String>,
    complete: AtomicBool,
}

impl SweepAudit {
    pub(super) fn new(
        root: &Path,
        config: &NoMistakesConfig,
        rule: &RuleDef,
        files: &[PathBuf],
        sources: &SourceStore,
        options: &Options,
        compiled: &CompiledOptions,
    ) -> anyhow::Result<Self> {
        let selected = super::super::selected_rule_files(root, config, rule, files)?;
        let selected: crate::fx::FxHashSet<_> = selected
            .iter()
            .filter(|path| compiled.includes(&relative_slash_path(root, path)))
            .collect();
        let (universe, _) = super::super::selected_files::rule_scope_files_with_roots(
            root,
            config,
            rule,
            &sources.inventory().paths(),
        );
        let sql_paths =
            crate::codebase::postgres::postgres_sql_paths(root, &universe, &compiled.schema)?;
        let sql = crate::codebase::check_facts::PathMembership::new(&sql_paths);
        let expected: Vec<_> = universe
            .iter()
            .filter(|path| {
                sql.contains(path) || crate::codebase::dependencies::extract::is_indexable(path)
            })
            .collect();
        let complete = !expected.is_empty()
            && compiled.shapes.keyset_only_sweep
            && config.effective_rule_analysis(rule)?.effort != crate::config::v2::Effort::Low
            && expected.into_iter().all(|path| selected.contains(path));
        let selectors = options
            .shape_options
            .keyset_only_sweep
            .ignore_tables
            .clone();
        Ok(Self {
            audit: AllowAudit::new(
                root,
                config,
                rule,
                Some(sources),
                "shapeOptions.keysetOnlySweep.ignoreTables",
                selectors.len(),
            ),
            selectors,
            complete: AtomicBool::new(complete),
        })
    }

    pub(super) fn finish(&self) -> Vec<RuleFinding> {
        self.audit.finish(
            |_| self.complete.load(Ordering::Relaxed),
            |index| self.selectors[index].clone(),
        )
    }
}

pub(super) fn observe_statements(opts: &CompiledOptions, file: &SqlStatementFileFacts) {
    if file.parse_failed || !file.parse_failures.is_empty() {
        incomplete(opts);
    }
}

pub(super) fn observe_embedded(
    opts: &CompiledOptions,
    file: &EmbeddedSqlFileFacts,
    facts: &crate::codebase::check_facts::CheckFactMap,
) {
    if facts
        .ts
        .get(&file.path)
        .is_none_or(|facts| facts.parse_error.is_some())
        || file.calls.iter().any(|call| call.is_unanalyzable())
        || file.fragments.iter().enumerate().any(|(index, fragment)| {
            fragment.sql_text.is_none()
                && file.fragment_variants.get(index).is_none_or(Vec::is_empty)
        })
    {
        incomplete(opts);
    }
}

fn incomplete(opts: &CompiledOptions) {
    if let Some(audit) = &opts.iteration.audit {
        audit.complete.store(false, Ordering::Relaxed);
    }
}

pub(super) fn observe_source(opts: &CompiledOptions, source: Option<&str>) {
    if source.is_none() {
        incomplete(opts);
    }
}
