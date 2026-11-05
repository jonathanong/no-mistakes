use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::SourceStore;
use crate::config::source_locations::{ConfigLocations, ConfigPath};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Default)]
pub(super) struct Locations {
    by_file: BTreeMap<String, Option<ConfigLocations>>,
}

impl Locations {
    pub(super) fn locate(
        &mut self,
        root: &Path,
        sources: &SourceStore,
        findings: &mut [RuleFinding],
    ) {
        for finding in findings {
            let (file, job) = finding
                .file
                .split_once('#')
                .map_or((finding.file.as_str(), None), |(file, job)| {
                    (file, Some(job))
                });
            let document = self.by_file.entry(file.to_owned()).or_insert_with(|| {
                let path = root.join(file);
                sources
                    .read_path(&path)
                    .ok()
                    .and_then(|source| ConfigLocations::parse(&source, &path))
            });
            let path = job.map_or_else(ConfigPath::default, |job| {
                ConfigPath::default().key("jobs").key(job)
            });
            finding.line = document
                .as_ref()
                .and_then(|locations| locations.line(&path))
                .unwrap_or(1);
            finding.file = file.to_owned();
        }
    }
}
