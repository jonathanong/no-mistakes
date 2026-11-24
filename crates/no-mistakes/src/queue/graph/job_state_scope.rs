use crate::config::v2::schema::QueuesTopLevelConfig;
use crate::queue::extract::FileFacts;
use crate::queue::graph_model::build_filter;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn filter_sites(
    root: &Path,
    config: &QueuesTopLevelConfig,
    facts: &mut HashMap<PathBuf, FileFacts>,
) -> anyhow::Result<()> {
    if let Some(policy) = &config.job_state_as_record {
        let patterns = policy
            .allow_paths
            .iter()
            .map(|pattern| pattern.trim_start_matches("./").to_owned())
            .collect::<Vec<_>>();
        if let Some(allowed) = build_filter(&patterns)? {
            for (path, file) in facts {
                if allowed.is_match(path.strip_prefix(root).unwrap_or(path)) {
                    file.job_state_sites.clear();
                }
            }
        }
    } else {
        for file in facts.values_mut() {
            file.job_state_sites.clear();
        }
    }
    Ok(())
}
