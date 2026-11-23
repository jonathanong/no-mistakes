use super::{check_with_files, finding, RuleFinding, RULE_ID};
use crate::codebase::rules::filesystem_dispatch::{
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts,
    PreparedFilesystemRuleInputs,
};
use crate::codebase::structured_value::parse_yaml_documents;
use crate::codebase::ts_source::{SourceStore, VisiblePathSnapshot};
use crate::config::v2::{load_v2_config, NoMistakesConfig};
use std::path::{Path, PathBuf};
use std::sync::Arc;

fn fixture_root(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/ast-grep-source-reuse")
        .join(name)
        .canonicalize()
        .expect("saved YAML fixture exists")
}

fn run_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    snapshot: &VisiblePathSnapshot,
    sources: &Arc<SourceStore>,
) -> Vec<RuleFinding> {
    run_filesystem_rules_with_config_snapshot_catalog_sources_and_facts(
        root,
        config,
        files,
        PreparedFilesystemRuleInputs {
            session: None,
            snapshot,
            next_config_resolver: None,
            vitest_catalog: None,
            sources: Arc::clone(sources),
            workflow_documents: None,
            tsconfig_gate_project_inputs: None,
            config_path: None,
        },
        None,
    )
    .unwrap()
}

#[test]
fn prepared_applications_read_and_parse_each_selected_yaml_file_once() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .unwrap();
    for (fixture, application_count) in [("repeated-12", 12), ("repeated-20", 20), ("single", 1)] {
        let root = fixture_root(fixture);
        let config = load_v2_config(&root, None).unwrap();
        assert_eq!(config.rule_applications(RULE_ID).len(), application_count);
        let files: Vec<_> = [
            "rules/files.yml",
            "rules/reports.yml",
            "rules/catalog.yml",
            "rules/sessions.yml",
            "rules/events.yml",
            "rules/metrics.yml",
            "rules/flags.yml",
            "rules/audit.yml",
            "rules/jobs.yml",
            "rules/cache.yml",
            "src/compat/adapter.ts",
            "src/generated/registry.ts",
        ]
        .into_iter()
        .map(|path| root.join(path))
        .collect();
        assert_eq!(files.len(), 12);
        let snapshot = VisiblePathSnapshot::from_tracked_paths(&root, &files);
        let sources = snapshot.source_store_for(&root);
        let findings = pool.install(|| run_prepared(&root, &config, &files, &snapshot, &sources));
        assert!(findings.is_empty(), "{fixture}: {findings:?}");
        assert_eq!(sources.physical_read_count(), 10, "{fixture}");
        assert_eq!(
            sources.structured_yaml_initialization_count(),
            10,
            "{fixture}"
        );
    }
}

#[test]
fn prepared_report_preserves_merge_keys_partial_documents_and_strict_read_errors() {
    let root = fixture_root("semantics");
    let config = load_v2_config(&root, None).unwrap();
    let relative = "rules/stream.yml";
    let invalid = "rules/invalid.yml";
    let stream_path = root.join(relative);
    let invalid_path = root.join(invalid);
    let files = [stream_path.clone(), invalid_path.clone()];
    let snapshot = VisiblePathSnapshot::from_tracked_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);

    let text = sources.read_path(&stream_path).unwrap();
    let old_documents = parse_yaml_documents(&text);
    assert_eq!(old_documents.values.len(), 2);
    assert_eq!(old_documents.errors.len(), 1);
    let mut expected: Vec<_> = old_documents
        .errors
        .into_iter()
        .map(|(line, message)| finding(relative, line, message, None))
        .collect();
    for (line, pattern) in [(5, "missing-merged.ts"), (13, "missing-explicit.ts")] {
        expected.push(finding(
            relative,
            line,
            format!("ast-grep ignore `{pattern}` matches no tracked path. Delete the entry or restore a tracked file that matches it."),
            Some(pattern.to_string()),
        ));
    }
    let error = std::fs::read_to_string(&invalid_path).unwrap_err();
    expected.push(finding(
        invalid,
        1,
        format!("failed to read `{invalid}` ({error})."),
        None,
    ));
    expected.sort();

    let prepared = run_prepared(&root, &config, &files, &snapshot, &sources);
    assert_eq!(prepared, expected);
    assert_eq!(sources.physical_read_count(), 2);
    assert_eq!(sources.structured_yaml_initialization_count(), 2);

    let (source, documents) = sources.parse_structured_yaml_path(&stream_path).unwrap();
    let (alias_source, alias_documents) = sources
        .parse_structured_yaml_path(&root.join("rules/../rules/stream.yml"))
        .unwrap();
    assert!(Arc::ptr_eq(&source, &text));
    assert!(Arc::ptr_eq(&source, &alias_source));
    assert!(Arc::ptr_eq(&documents, &alias_documents));
    assert_eq!(sources.structured_yaml_initialization_count(), 2);

    let standalone = check_with_files(&root, &config, &files).unwrap();
    assert_eq!(standalone, expected);
}

#[test]
fn supplied_missing_file_reports_the_original_io_error_and_memoizes_failure() {
    let root = fixture_root("semantics");
    let path = root.join("rules/missing.yml");
    let snapshot = VisiblePathSnapshot::from_tracked_paths(&root, std::slice::from_ref(&path));
    let sources = snapshot.source_store_for(&root);
    let error = std::fs::read_to_string(&path).unwrap_err();
    for _ in 0..2 {
        let cached = sources.parse_structured_yaml_path(&path).err().unwrap();
        assert_eq!(cached.kind(), error.kind());
        assert_eq!(cached.to_string(), error.to_string());
    }
    assert_eq!(sources.physical_read_count(), 1);
    assert_eq!(sources.structured_yaml_initialization_count(), 1);
}
