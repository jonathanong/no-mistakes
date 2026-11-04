use super::{load_v2_config_with_path_from_visible, parse_sources_quiet, ConfigDocument};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/config-fragments")
        .join(name)
}

#[test]
fn split_rules_concatenate_in_path_order() {
    let root = fixture("valid");
    let visible = [
        ".no-mistakes.yml",
        ".no-mistakes/rules/nested/extra.yml",
        ".no-mistakes/checks/a.yml",
    ]
    .map(|p| root.join(p));
    let (config, _) = load_v2_config_with_path_from_visible(&root, None, &visible).unwrap();
    assert_eq!(config.rules.len(), 2);
}

#[test]
fn historical_documents_reject_unknown_section() {
    let root = fixture("invalid");
    let document = ConfigDocument {
        path: root.join(".no-mistakes/rule/a.yml"),
        section: Some("rule".into()),
        source: std::fs::read_to_string(root.join(".no-mistakes/rule/a.yml")).unwrap(),
    };
    assert!(parse_sources_quiet(&root, &[document])
        .unwrap_err()
        .to_string()
        .contains("unknown config section"));
}

#[test]
fn mismatched_fragment_types_report_both_files() {
    let root = fixture("conflict");
    let documents = [
        ConfigDocument {
            path: root.join(".no-mistakes.yml"),
            section: None,
            source: std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap(),
        },
        ConfigDocument {
            path: root.join(".no-mistakes/checks/a.yml"),
            section: Some("checks".into()),
            source: std::fs::read_to_string(root.join(".no-mistakes/checks/a.yml")).unwrap(),
        },
    ];
    let error = parse_sources_quiet(&root, &documents)
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes.yml") && error.contains("checks/a.yml"));
}
