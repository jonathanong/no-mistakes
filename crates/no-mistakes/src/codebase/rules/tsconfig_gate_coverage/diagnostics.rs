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
mod tests {
    #[test]
    fn stale_allowlist_entries_point_to_the_fragment_entry() {
        let root = crate::codebase::ts_resolver::normalize_path(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../test-cases/config-v2/split-provenance/fixture/rule-diagnostics"),
        );
        let config = crate::config::v2::load_v2_config(&root, None).unwrap();
        let rule = config
            .rules
            .iter()
            .find(|rule| rule.rule == super::super::RULE_ID)
            .unwrap();
        let options = rule.try_rule_options().unwrap();
        let empty = std::collections::BTreeSet::new();
        let mut findings = super::super::scan_application(
            &options,
            &empty,
            &empty,
            &empty,
            &empty,
            &empty,
            ".no-mistakes.yml",
        );
        let sources = crate::codebase::rules::source_store_for_files(&[]);
        super::annotate(&root, &config, rule, &sources, &mut findings);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, ".no-mistakes/rules/types.yml");
        assert_eq!(findings[0].line, 4);
    }
}
