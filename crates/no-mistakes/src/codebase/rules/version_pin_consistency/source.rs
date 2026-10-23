use super::{parse::parse_source, path::read_text, scan::finding, Anchor, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use serde_yaml::Value;
use std::{path::Path, sync::Arc};

pub(super) fn load(
    root: &Path,
    rel: &str,
    selected: bool,
    anchors: &[&Anchor],
    sources: &SourceStore,
) -> Result<(Arc<str>, Value), Vec<RuleFinding>> {
    let text = read_text(root, rel, sources).map_err(|error| incomplete(rel, anchors, &error))?;
    match parse_source(Path::new(rel), &text) {
        Ok(value) => Ok((text, value)),
        // Preserve established validation of a selected source itself.
        Err(error) if selected => Err(vec![finding(rel, 1, format!("{rel}: {error}"))]),
        Err(error) => Err(incomplete(rel, anchors, &error)),
    }
}

fn incomplete(source: &str, anchors: &[&Anchor], reason: &str) -> Vec<RuleFinding> {
    anchors.iter().map(|anchor| {
        let rel = super::parse::configured_rel(&anchor.file);
        let mut result = finding(rel, 1, format!(
            "{rel}: version comparison is incomplete because source {source} could not be analyzed ({reason}). Restore the declared source file and correct its syntax so the anchor pin can be compared."
        ));
        result.target = Some("unanalyzable".to_owned());
        result
    }).collect()
}
