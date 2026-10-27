use super::*;
use crate::codebase::ts_source::disable_directives;

#[test]
fn known_catalog_covers_every_declared_rule_and_registered_executor_rule() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/codebase/rules");
    let declarations =
        regex::Regex::new(r#"pub const (?:[A-Z_]*RULE_ID): &str =\s*"([^"]+)";"#).unwrap();
    let modules = regex::Regex::new(r"(?m)^pub mod ([a-z_]+);").unwrap();
    let mut declared = BTreeSet::new();
    // Follow the explicit module inventory, not a recursive filesystem walk.
    // This independently catches new core rules missing the manual audit list.
    for module in modules.captures_iter(include_str!("../mod.rs")) {
        let leaf = directory.join(format!("{}.rs", &module[1]));
        let path = if leaf.is_file() {
            leaf
        } else {
            directory.join(&module[1]).join("mod.rs")
        };
        let source = std::fs::read_to_string(path).unwrap();
        for declaration in declarations.captures_iter(&source) {
            declared.insert(declaration[1].to_owned());
        }
    }
    assert!(
        !declared.is_empty(),
        "the rule declaration inventory must not become empty"
    );
    for rule in declared
        .iter()
        .map(String::as_str)
        .chain(
            crate::config::v2::schema::postgres_executors::RULE_IDS
                .iter()
                .copied(),
        )
        .chain(
            super::super::filesystem_dispatch::FILESYSTEM_RULE_IDS
                .iter()
                .copied(),
        )
    {
        assert!(
            known_rule(rule),
            "registered rule `{rule}` is missing from the suppression catalog"
        );
    }
    let domain_declarations =
        regex::Regex::new(r#"pub const [A-Z_]+: &str =\s*"([^"]+)";"#).unwrap();
    for declaration in domain_declarations.captures_iter(concat!(
        include_str!("../../../playwright/rules.rs"),
        include_str!("../../unique_exports.rs")
    )) {
        assert!(
            known_rule(&declaration[1]),
            "domain rule `{}` is missing from the suppression catalog",
            &declaration[1]
        );
    }
    // Aggregate adapters emit these names without a public RULE_ID constant.
    for rule in [
        "assert-no-fetch",
        "queues-check",
        "unused-disable-directive",
        "invalid-disable-directive",
    ] {
        assert!(known_rule(rule));
    }
    assert!(!known_rule("typo-configured-as-a-rule"));
}

fn recent_fixture() -> (PathBuf, Vec<PathBuf>, NoMistakesConfig) {
    let root = super::tests::root();
    let files = vec![root.join("recent-rules.mts")];
    let config = serde_yaml::from_str(include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/recent-rules.yml"
    ))
    .unwrap();
    (root, files, config)
}

#[test]
fn recent_rules_are_disabled_rather_than_unknown_when_unconfigured() {
    let (root, files, mut config) = recent_fixture();
    for rule in &mut config.rules {
        rule.enabled = false;
    }
    let sources = super::super::source_store_for_files(&files);
    let findings = audit(&root, &config, &files, &sources, &[]).unwrap();
    assert_eq!(findings.len(), 5);
    for finding in findings {
        assert_eq!(finding.rule, "invalid-disable-directive");
        if finding.target.as_deref() == Some("typo-configured-as-a-rule") {
            assert!(finding.message.contains("unknown rule"));
        } else {
            assert!(finding.message.contains("disabled or out-of-scope rule"));
        }
    }
}

#[test]
fn recent_rules_consumed_suppressions_are_used_and_shadowed_comments_remain_unused() {
    let (root, files, config) = recent_fixture();
    let sources = super::super::source_store_for_files(&files);
    let mut findings: Vec<_> = config
        .rules
        .iter()
        .map(|rule| RuleFinding {
            rule: rule.rule.clone(),
            file: "recent-rules.mts".into(),
            line: 6,
            message: "actual violation".into(),
            import: None,
            target: None,
        })
        .collect();
    let suppressed = super::super::suppress_domain_findings_with_sources(
        &root,
        &mut findings,
        &sources,
        |finding| super::super::SuppressionTarget {
            domain: "rules",
            rule: &finding.rule,
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    );
    assert!(findings.is_empty());
    assert_eq!(suppressed.len(), 2);
    let audit = audit(&root, &config, &files, &sources, &suppressed).unwrap();
    assert_eq!(audit.len(), 3);
    assert_eq!(
        audit.iter().map(|finding| finding.line).collect::<Vec<_>>(),
        [3, 4, 5]
    );
    assert_eq!(audit[2].rule, "unused-disable-directive");
    assert!(audit[0].message.contains("unknown rule"));
    assert!(audit[1].message.contains("disabled or out-of-scope rule"));
    assert_eq!(sources.physical_read_count(), 1);
}

#[test]
fn comma_separated_rules_do_not_suppress_either_rule_in_any_directive_form() {
    let source = include_str!(
        "../../../../../../fixtures/rules/disable-directive-audit/comma-separated.mts"
    );
    let directives = disable_directives(source, true);
    assert_eq!(directives.len(), 3);
    // A comma is part of an unknown name, never a list of supported rule IDs.
    assert!(directives
        .iter()
        .all(|directive| directive.rule
            == "postgres-no-test-state-mutation,forbidden-lint-suppressions"));
    assert!(directives
        .iter()
        .all(|directive| !known_rule(&directive.rule)));
    for rule in [
        "postgres-no-test-state-mutation",
        "forbidden-lint-suppressions",
    ] {
        assert!(!crate::codebase::ts_source::has_disable_file_comment(
            source, rule
        ));
        assert!(!crate::codebase::ts_source::has_disable_comment(
            source, 3, rule
        ));
        assert!(!crate::codebase::ts_source::has_disable_line_comment(
            source, 3, rule
        ));
    }
}

#[test]
fn rule_local_options_do_not_hide_unused_directives_in_declared_application_scope() {
    let (root, files, mut config) = recent_fixture();
    // These rule-local selectors deliberately select no analyzer input. The
    // audit still checks the enclosing application's declared include scope.
    config.rules[0].options =
        serde_yaml::from_str("exclude: [recent-rules.mts]\nsqlInclude: [missing.sql]").unwrap();
    let sources = super::super::source_store_for_files(&files);
    let findings = audit(&root, &config, &files, &sources, &[]).unwrap();
    let unused: Vec<_> = findings
        .iter()
        .filter(|finding| finding.rule == "unused-disable-directive")
        .map(|finding| finding.line)
        .collect();
    assert_eq!(unused, [1, 2, 5]);
}
