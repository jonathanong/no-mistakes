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

#[test]
fn invalid_fragment_globs_point_to_the_authored_line() {
    let root = fixture("invalid-glob");
    let path = root.join(".no-mistakes/projects/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(".no-mistakes/projects/bad.yml:3:"),
        "{error}"
    );
    assert!(
        error.contains("projects.app.include[0]: contains invalid glob `[`"),
        "{error}"
    );
}

#[test]
fn fragment_validation_preserves_non_glob_errors() {
    let root = fixture("invalid-rule");
    let path = root.join(".no-mistakes/rules/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert_eq!(error, "rules[0].rule is required");
}

#[test]
fn typed_sequence_errors_point_to_the_fragment_item() {
    let root = fixture("invalid-sequence");
    let path = root.join(".no-mistakes/rules/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes/rules/bad.yml:3:"), "{error}");
    assert!(error.contains("rules[0].include[0]"), "{error}");
}

#[test]
fn relative_visible_fragment_paths_are_rooted_and_selected() {
    let root = fixture("valid");
    let visible = [PathBuf::from(".no-mistakes/rules/nested/extra.yml")];
    let selected = super::fragments::select_fragments(&root, &visible).unwrap();
    assert_eq!(selected, [(root.join(&visible[0]), "rules".into())]);
}

#[test]
fn malformed_rules_fragment_keeps_the_typed_shape_error() {
    let root = fixture("rules-mapping");
    let path = root.join(".no-mistakes/rules/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(error.contains(".no-mistakes/rules/bad.yml:1:"), "{error}");
    assert!(
        error.contains("rules: invalid type: map, expected a sequence"),
        "{error}"
    );
}

#[test]
fn non_string_fragment_keys_preserve_typed_errors() {
    let root = fixture("non-string-key");
    let path = root.join(".no-mistakes/projects/bad.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(".no-mistakes/projects/bad.yml:1:"),
        "{error}"
    );
    assert!(error.contains("expected a string"), "{error}");
}

#[test]
fn vitest_setup_declaration_paths_keep_fragment_provenance() {
    let root = fixture("vitest-setup-provenance");
    let visible = [
        PathBuf::from(".no-mistakes.yml"),
        PathBuf::from(".no-mistakes/tests/vitest.yml"),
    ];
    let (config, _) = load_v2_config_with_path_from_visible(&root, None, &visible).unwrap();
    let policy = &config.tests.vitest.projects["app"];
    let fragment =
        crate::codebase::ts_resolver::normalize_path(&root.join(".no-mistakes/tests/vitest.yml"));
    assert_eq!(
        policy.setup_declaration_paths,
        vec![fragment.clone(), fragment.clone()]
    );
    assert_eq!(policy.declaration_path.as_deref(), Some(fragment.as_path()));
}

#[test]
fn invalid_project_enum_reports_its_fragment_line() {
    let root = fixture("invalid-project-enum");
    let path = root.join(".no-mistakes/projects/app.yml");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(".no-mistakes/projects/app.yml:2:"),
        "{error}"
    );
    assert!(error.contains("unknown variant `impossible`"), "{error}");
}

#[test]
fn historical_documents_reject_mismatched_section_metadata() {
    let root = fixture("valid");
    let path = root.join(".no-mistakes/rules/nested/extra.yml");
    let document = ConfigDocument {
        source: std::fs::read_to_string(&path).unwrap(),
        path,
        section: Some("checks".into()),
    };
    let error = parse_sources_quiet(&root, &[document])
        .unwrap_err()
        .to_string();
    assert!(error.contains("incorrect config section"), "{error}");
}

#[test]
fn historical_ignored_documents_do_not_contribute_config() {
    let root = fixture("cases");
    let documents: Vec<_> = ["readme.md", ".ignored"]
        .into_iter()
        .map(|name| {
            let path = root.join(".no-mistakes/rules").join(name);
            ConfigDocument {
                source: std::fs::read_to_string(&path).unwrap(),
                path,
                section: Some("rules".into()),
            }
        })
        .collect();
    let config = parse_sources_quiet(&root, &documents).unwrap();
    assert!(config.rules.is_empty());
}

#[test]
fn duplicate_relation_keys_fail_before_fragment_deserialization() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/config-v2/split-provenance/fixture/duplicate-json");
    let path = root.join(".no-mistakes/rules/policy.json");
    let error = load_v2_config_with_path_from_visible(&root, None, &[path])
        .unwrap_err()
        .to_string();
    assert!(
        error.contains(".no-mistakes/rules/policy.json:5:"),
        "{error}"
    );
    assert!(error.contains("duplicate key accountId"), "{error}");
}
