use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn standalone_rule_checks_the_saved_policies_and_respects_suppression() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &configured(None),
        &[
            root.join("policies/receipt.tf"),
            root.join("policies/justified.tf"),
            root.join("policies/generated.tf"),
        ],
    )
    .unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [4, 10, 19]);
}

#[test]
fn standalone_rule_routes_unanalyzable_inputs_through_policy() {
    let root = fixture_root();
    let files = [
        root.join("policies/accepted.tf"),
        root.join("policies/example.tf.json"),
        root.join("policies/receipt.tf"),
    ];
    let sources = super::super::source_store_for_files(&files);

    let reported = diagnostics_prepared(&root, &configured(None), &files, &sources, None, false)
        .unwrap();
    assert!(reported.advisories.is_empty());
    let reported_locations: Vec<_> = reported
        .findings
        .iter()
        .map(|f| (f.file.as_str(), f.line, f.target.as_deref()))
        .collect();
    assert_eq!(
        reported_locations,
        [
            ("policies/accepted.tf", 12, Some("unanalyzable")),
            ("policies/accepted.tf", 18, Some("unanalyzable")),
            ("policies/accepted.tf", 22, Some("unanalyzable")),
            ("policies/accepted.tf", 32, Some("unanalyzable")),
            ("policies/accepted.tf", 62, Some("unanalyzable")),
            ("policies/accepted.tf", 75, Some("unanalyzable")),
            ("policies/accepted.tf", 79, Some("unanalyzable")),
            ("policies/example.tf.json", 1, Some("unanalyzable")),
            ("policies/receipt.tf", 4, None),
            ("policies/receipt.tf", 10, None),
            ("policies/receipt.tf", 19, None),
        ]
    );

    let advised = diagnostics_prepared(
        &root,
        &configured(Some("advise")),
        &files,
        &sources,
        None,
        false,
    )
    .unwrap();
    assert_eq!(advised.findings.len(), 3);
    assert_eq!(advised.advisories.len(), 8);

    let ignored = diagnostics_prepared(
        &root,
        &configured(Some("ignore")),
        &files,
        &sources,
        None,
        false,
    )
    .unwrap();
    assert_eq!(ignored.findings.len(), 3);
    assert!(ignored.advisories.is_empty());
}

#[test]
fn standalone_rule_reports_iam_role_inline_policies_as_unanalyzable() {
    let root = fixture_root();
    let files = [root.join("outside/inline_role.tf")];
    let sources = super::super::source_store_for_files(&files);
    let reported =
        diagnostics_prepared(&root, &configured(None), &files, &sources, None, false).unwrap();
    let locations: Vec<_> = reported
        .findings
        .iter()
        .map(|f| (f.file.as_str(), f.line, f.target.as_deref()))
        .collect();
    assert_eq!(
        locations,
        [
            ("outside/inline_role.tf", 6, Some("unanalyzable")),
            ("outside/inline_role.tf", 16, Some("unanalyzable")),
        ]
    );
}

#[test]
fn standalone_rule_reports_the_path_of_unreadable_or_malformed_input() {
    let root = fixture_root();
    for file in ["incomplete/missing.tf", "incomplete/broken.tf"] {
        let error = check_with_files(&root, &configured(None), &[root.join(file)]).unwrap_err();
        let failure = error.downcast_ref::<CheckInputFailure>().unwrap();
        assert_eq!(failure.file.as_deref(), Some(Path::new(file)));
    }
}

fn configured(unanalyzable: Option<&str>) -> NoMistakesConfig {
    let options = match unanalyzable {
        Some(mode) => serde_yaml::from_str(&format!("unanalyzable: {mode}")).unwrap(),
        None => serde_yaml::Value::Null,
    };
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            options,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(RULE_ID))
}
