use super::*;
use crate::codebase::rules::analysis_policy::prepare_rule_analysis;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

fn effort_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/test-no-unmocked-dynamic-imports/fixture/effort-low"),
    )
}

fn marked_lines(source: &str, marker: &str) -> Vec<usize> {
    source
        .lines()
        .enumerate()
        .filter_map(|(index, line)| line.contains(marker).then_some(index + 1))
        .collect()
}

#[test]
fn standalone_prepared_check_honors_setup_shape_disable_comments() {
    let root = effort_root();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let mut config =
        crate::config::v2::load_v2_config(&root, Some(&root.join(".no-mistakes-report.yml")))
            .unwrap();
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    prepare_rule_analysis(&root, &mut config, &files, &sources).unwrap();
    let shared = crate::codebase::check_facts::collect_check_facts(
        &root,
        files.clone(),
        crate::codebase::check_facts::CheckFactPlan {
            imports: true,
            dynamic_imports: true,
            source: true,
            ..Default::default()
        },
    );
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, &files).unwrap();
    let findings = check_with_prepared_facts(&root, &config, &tsconfig, &shared).unwrap();
    let setup = std::fs::read_to_string(root.join("src/setup.mts")).unwrap();
    let reported = findings
        .iter()
        .filter(|finding| {
            finding.file == "src/setup.mts"
                && finding.message.contains("unanalyzable at effort low")
        })
        .map(|finding| finding.line)
        .collect::<Vec<_>>();
    assert_eq!(reported, marked_lines(&setup, "EXPECT_SHAPE"));
    for line in marked_lines(&setup, "suppressed-dynamic") {
        assert!(
            findings
                .iter()
                .all(|finding| finding.file != "src/setup.mts" || finding.line != line),
            "disabled setup shape must stay silent: {findings:?}"
        );
    }
}
