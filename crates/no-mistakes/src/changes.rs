use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use crate::codebase::keyed_diff::KeyChanges;
use crate::config::v2::load_v2_config_from_source_store;

pub mod config;
mod file_diff;
mod report;
mod rules;
mod snapshot;
mod wire;

use file_diff::compare;
use report::ReportBuilder;
use rules::Rules;
use snapshot::Snapshot;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChangesDiffArgs {
    pub base_root: PathBuf,
    pub head_root: PathBuf,
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", try_from = "wire::WireReport")]
pub struct ChangesReport {
    pub schema_version: u32,
    #[serde(flatten)]
    pub categories: BTreeMap<String, KeyChanges>,
    pub unclassified: Vec<String>,
    pub complete: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reasons: Vec<String>,
}

impl ChangesReport {
    fn new(reasons: Vec<String>) -> Self {
        Self {
            schema_version: 1,
            categories: BTreeMap::new(),
            unclassified: Vec::new(),
            complete: reasons.is_empty(),
            reasons,
        }
    }

    fn finish(mut self) -> Self {
        sort_unique(&mut self.unclassified);
        sort_unique(&mut self.reasons);
        self.complete = self.reasons.is_empty();
        self
    }
}

pub fn diff(args: &ChangesDiffArgs) -> Result<ChangesReport> {
    let (base, head) = crate::perf_trace::trace("changes.diff.discover", || {
        rayon::join(
            || Snapshot::discover(&args.base_root, "base"),
            || Snapshot::discover(&args.head_root, "head"),
        )
    });
    let (base, head) = match (base, head) {
        (Ok(base), Ok(head)) => (base, head),
        (base, head) => {
            let reasons = base.err().into_iter().chain(head.err()).collect();
            return Ok(ChangesReport::new(reasons).finish());
        }
    };
    let reasons: Vec<_> = base.reasons.iter().chain(&head.reasons).cloned().collect();
    let config = match load_v2_config_from_source_store(
        &head.root,
        args.config.as_deref(),
        &head.sources.inventory().target_file_paths(),
        &head.sources,
    ) {
        Ok(config) => config,
        Err(_) if !reasons.is_empty() => return Ok(ChangesReport::new(reasons).finish()),
        Err(error) => return Err(error),
    };
    let rules = Rules::compile(&config.changes)?;
    let paths: BTreeSet<_> = base.paths.keys().chain(head.paths.keys()).collect();
    let mut report = ReportBuilder::new(config.changes.categories.keys(), reasons);
    for diff in crate::perf_trace::trace("changes.diff.compare", || {
        paths
            .into_par_iter()
            .filter_map(|path| compare(path, &base, &head, &rules))
            .collect::<Vec<_>>()
    }) {
        report.record(diff);
    }
    Ok(report.finish())
}

fn sort_unique(values: &mut Vec<String>) {
    values.sort_unstable();
    values.dedup();
}
