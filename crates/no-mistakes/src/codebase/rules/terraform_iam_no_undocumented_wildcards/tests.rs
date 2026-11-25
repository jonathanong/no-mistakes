use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

#[test]
fn standalone_rule_checks_the_saved_policies_and_respects_suppression() {
    let root = fixture_root();
    let findings = check_with_files(
        &root,
        &configured(),
        &[
            root.join("policies/receipt.tf"),
            root.join("policies/justified.tf"),
            root.join("policies/generated.tf"),
        ],
    )
    .unwrap();
    let lines: Vec<_> = findings.iter().map(|finding| finding.line).collect();
    assert_eq!(lines, [4, 10]);
}

#[test]
fn standalone_rule_reports_the_path_of_unreadable_or_malformed_input() {
    let root = fixture_root();
    for file in ["incomplete/missing.tf", "incomplete/broken.tf"] {
        let error = check_with_files(&root, &configured(), &[root.join(file)]).unwrap_err();
        let failure = error.downcast_ref::<CheckInputFailure>().unwrap();
        assert_eq!(failure.file.as_deref(), Some(Path::new(file)));
    }
}

fn configured() -> NoMistakesConfig {
    NoMistakesConfig {
        rules: vec![RuleDef {
            rule: RULE_ID.into(),
            scope: Some(RuleScope::Repository),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn fixture_root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(&crate::test_support::rule_fixture_root(RULE_ID))
}
