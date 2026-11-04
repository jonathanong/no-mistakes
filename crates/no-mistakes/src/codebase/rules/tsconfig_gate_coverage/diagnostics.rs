use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::source_locations::ConfigPath;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use std::path::Path;

pub(super) fn config_file(root: &Path, path: Option<&Path>) -> String {
    path.map_or_else(
        || ".no-mistakes.yml".to_string(),
        |path| {
            relative_slash_path(
                &crate::codebase::ts_resolver::normalize_path(root),
                &crate::codebase::ts_resolver::normalize_path(path),
            )
        },
    )
}

pub(super) fn annotate(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &RuleDef,
    sources: &SourceStore,
    findings: &mut [RuleFinding],
) {
    if config.config_provenance.is_empty() {
        return;
    }
    let Some(index) = config
        .rules
        .iter()
        .position(|entry| std::ptr::eq(entry, rule))
    else {
        return;
    };
    for finding in findings
        .iter_mut()
        .filter(|finding| finding.target.is_some())
    {
        let origin = ConfigPath::default()
            .key("rules")
            .index(index)
            .key("options")
            .key("allowProjects")
            .key(finding.target.as_deref().unwrap());
        if let Some((path, local)) = config.config_provenance.origin(&origin) {
            finding.file = relative_slash_path(root, path);
            finding.line = sources
                .config_locations(path)
                .and_then(|locations| locations.line(&local))
                .unwrap_or(1);
        }
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
