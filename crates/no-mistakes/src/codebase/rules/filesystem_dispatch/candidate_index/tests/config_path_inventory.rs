use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn config_path_references_keeps_tracked_skip_dir_inventory() {
    let root = crate::codebase::ts_resolver::normalize_path(Path::new(env!("CARGO_MANIFEST_DIR")));
    let tracked = root.join("knip.json");
    let skipped = root.join("fixtures/direct-import.mts");
    let untracked = root.join("generated.json");
    let files = vec![tracked.clone(), untracked];
    let tracked_files = vec![tracked.clone()];
    let inventory = Arc::new(vec![skipped.clone(), tracked.clone()]);
    let repository_rule = |rule: &str| RuleDef {
        rule: rule.to_string(),
        scope: Some(RuleScope::Repository),
        ..Default::default()
    };
    let config = NoMistakesConfig {
        rules: vec![
            repository_rule(super::super::CONFIG_PATH_REFERENCES),
            repository_rule(super::super::NO_MISTAKES_CONFIG),
        ],
        ..Default::default()
    };

    let index = RuleCandidateIndex::prepare_with_inventory(
        &root,
        &config,
        &files,
        &tracked_files,
        &[],
        Some(inventory),
    )
    .unwrap();

    let mut expected = vec![skipped, tracked.clone()];
    expected.sort();
    assert_eq!(
        index.candidates(super::super::CONFIG_PATH_REFERENCES),
        expected,
        "path existence must see tracked files under source skip directories"
    );
    assert_eq!(
        index.candidates(super::super::NO_MISTAKES_CONFIG),
        std::slice::from_ref(&tracked),
        "unrelated tracked-only rules stay on the source universe"
    );
}
