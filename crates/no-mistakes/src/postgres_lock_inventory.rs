use crate::codebase::{analysis_session::AnalysisSession, rules::postgres_lock_inventory as rule};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

pub use rule::{
    LockInventoryBaseline, LockInventoryBaselineSite, LockInventoryReport, LockInventorySite,
    UnanalyzedLockSite,
};
#[cfg(test)]
mod tests;
mod write;

#[derive(Debug, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PostgresLockInventoryOptions {
    pub root: PathBuf,
    pub config: Option<PathBuf>,
    pub write_baseline: bool,
}

impl Default for PostgresLockInventoryOptions {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            config: None,
            write_baseline: false,
        }
    }
}

pub fn run(options: &PostgresLockInventoryOptions) -> Result<LockInventoryReport> {
    let root = crate::cli::resolve_root(&options.root, &std::env::current_dir()?);
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let session = AnalysisSession::new(crate::diagnostics::current());
    let visible = session.visible_paths(&root);
    let files = visible.paths_for(&root);
    let config = session.config(&root, options.config.as_deref())?;
    rule::standalone_application(&config)?;
    let sources = session
        .existing_sources_for(&root)
        .expect("discovered inventory owns sources");
    let facts = crate::codebase::postgres::prepare_rule_sql_facts(
        &root,
        &files,
        std::sync::Arc::clone(&sources),
        &config,
        &[rule::RULE_ID],
    )?;
    let mut report =
        rule::inventory_with_files_sources_and_facts(&root, &config, &files, &sources, &facts)?;
    if options.write_baseline {
        let path = root.join(&report.baseline_file);
        let existing = match sources.read_path(&path) {
            Ok(_) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error).context("read lock inventory baseline"),
        };
        anyhow::ensure!(
            existing
                || (!visible.git_index_paths_for(&root).contains(&path)
                    && !(visible.git_index_available_for(&root)
                        && write::staged_deletion(&root, &report.baseline_file)?)),
            "restore the tracked lock inventory baseline before rewriting it; a tracked baseline cannot be reinitialized"
        );
        anyhow::ensure!(
            report.unanalyzed.is_empty(),
            "cannot rewrite lock inventory baseline while SQL is unanalyzed"
        );
        anyhow::ensure!(!existing || report.added.is_empty(), "lock inventory baseline may only go down; remove newly added lock sites before rewriting it");
        let mut sites: Vec<_> = report
            .sites
            .iter()
            .map(|site| LockInventoryBaselineSite {
                key: site.key.clone(),
                file: site.file.clone(),
                category: site.category.clone(),
            })
            .collect();
        sites.sort();
        write::publish(
            &root,
            &report.baseline_file,
            &LockInventoryBaseline { version: 1, sites },
            existing,
        )?;
        report.added.clear();
        report.removed.clear();
        report.baseline_updated = true;
    }
    Ok(report)
}
