use std::path::PathBuf;
use std::process::{Command, Output};

const RULE: &str = "production-exports-only-used-by-tests";

fn check(scenario: &str) -> Output {
    check_config(scenario, ".no-mistakes.yml")
}

fn check_config(scenario: &str, config: &str) -> Output {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/production-exports-only-used-by-tests/fixture")
            .join(scenario),
    );
    Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--root"])
        .arg(&root)
        .arg("--config")
        .arg(root.join(config))
        .args(["--format", "json"])
        .output()
        .unwrap()
}

#[test]
fn computed_imports_preserve_definite_findings_and_failure_status_for_every_policy() {
    for policy in ["report", "advise", "ignore"] {
        let output = check_config("policy-isolation", &format!("{policy}.yml"));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{policy}: {report}");
        let rules = report["rules"].as_array().unwrap();
        let definite: Vec<_> = rules
            .iter()
            .filter(|row| row["target"] != "unanalyzable")
            .collect();
        assert_eq!(definite.len(), 1, "{policy}: {report}");
        assert_eq!(definite[0]["target"], "testOnly");
        assert_eq!(
            rules.iter().any(|row| row["target"] == "unanalyzable"),
            policy == "report"
        );
        assert_eq!(
            !report["advisories"].as_array().unwrap().is_empty(),
            policy == "advise"
        );
    }
}

#[test]
fn production_exports_test_only_reports_symbol_consumers_through_barrels() {
    let output = check("fail");
    let body = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{body}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["rules"].as_array().unwrap();
    for symbol in [
        "createLinkPost",
        "getRequestClientInfo",
        "createFlowProducer",
        "onlyTest",
    ] {
        assert!(
            findings
                .iter()
                .any(|finding| { finding["rule"] == RULE && finding.to_string().contains(symbol) }),
            "missing {symbol}: {body}"
        );
    }
    // Production consumption belongs to the selected symbol, not its entire file.
    assert!(!findings.iter().any(|finding| {
        finding["rule"] == RULE && finding.to_string().contains("realProduction")
    }));
}

#[test]
fn production_exports_test_only_ignores_production_consumers_and_suppressions() {
    for scenario in ["pass", "suppressed"] {
        let output = check(scenario);
        let body = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{scenario}: {body}");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert!(report["rules"].as_array().unwrap().is_empty(), "{body}");
    }
}

#[test]
fn selector_advisories_check_json_snapshot() {
    let output = check("selector-advisories");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{report}");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{report}");
    for target in ["testOnlyValue", "mockOnlyValue"] {
        assert!(
            findings.iter().any(|item| item["target"] == target),
            "{report}"
        );
    }
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 3, "{report}");
    for (file, line, specifier, other) in [
        (
            "other/candidate.mts",
            2,
            "../src/candidate.mjs",
            "../src/proven.mjs",
        ),
        ("other/pair.mts", 2, "../src/first.mjs", "../src/second.mjs"),
        ("other/pair.mts", 4, "../src/second.mjs", "../src/first.mjs"),
    ] {
        let advisory = advisories
            .iter()
            .find(|item| {
                item["file"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(file)
                    && item["line"] == line
            })
            .unwrap_or_else(|| panic!("missing advisory at {file}:{line}: {report}"));
        let detail = advisory.to_string();
        assert!(detail.contains(specifier), "{report}");
        assert!(!detail.contains(other), "{report}");
        assert!(detail.contains("namespace"), "{report}");
        if file == "other/candidate.mts" {
            assert!(detail.contains("other/candidate.mts:2"), "{report}");
            assert!(detail.contains("other/z-candidate.mts:2"), "{report}");
        }
    }
    let root = crate::check_json_snapshot::rules_path(RULE, "fixture", "selector-advisories");
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_selector_advisories",
        &[(
            "selector_advisories",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn mock_controls_check_json_snapshot() {
    let output = check("mock-controls");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{report}");
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    let advisories = report["advisories"].as_array().unwrap();
    // Each real module load can conceal production consumption.
    for (file, specifier) in [
        ("other/local.mts", "../src/local.mjs"),
        ("other/alias.mts", "../src/alias.mjs"),
        ("other/shared.mts", "../src/shared.mjs"),
        ("other/original.mts", "../src/original.mjs"),
        ("other/actual.mts", "../src/actual.mjs"),
        ("other/constructor.mts", "../src/constructor.mjs"),
        ("other/tagged.mts", "../src/tagged.mjs"),
        ("other/nested-import.mts", "../src/nested-import.mjs"),
        ("other/unbound.mts", "../src/unbound.mjs"),
        ("other/wrong-binding.mts", "../src/wrong-binding.mjs"),
        ("other/local-binding.mts", "../src/local-binding.mjs"),
        ("other/optional.mts", "../src/optional.mjs"),
        ("other/reference.mts", "../src/reference.mjs"),
        ("other/no-factory.mts", "../src/no-factory.mjs"),
        ("other/opaque-member.mts", "../src/opaque-member.mjs"),
        ("other/opaque-reference.mts", "../src/opaque-reference.mjs"),
        ("other/generator.mts", "../src/generator.mjs"),
        ("other/shadowed.mts", "../src/shadowed.mjs"),
        ("other/opaque-call.mts", "../src/opaque-call.mjs"),
        ("other/getter.mts", "../src/getter.mjs"),
        ("other/bare-callee.mts", "../src/bare-callee.mjs"),
        ("other/other-method.mts", "../src/other-method.mjs"),
        ("other/nested-callee.mts", "../src/nested-callee.mjs"),
        (
            "other/multiple-statements.mts",
            "../src/multiple-statements.mjs",
        ),
        ("other/spread.mts", "../src/spread.mjs"),
        (
            "other/captured-property.mts",
            "../src/captured-property.mjs",
        ),
    ] {
        assert!(
            advisories.iter().any(|item| {
                item["file"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(file)
                    && item["line"] == 3
                    && item.to_string().contains(specifier)
            }),
            "missing selector provenance for {file}: {report}"
        );
    }
    let root = crate::check_json_snapshot::rules_path(RULE, "fixture", "mock-controls");
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_mock_controls",
        &[("mock_controls", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn type_imports_check_json_snapshot() {
    let output = check("type-imports");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{report}");
    // Proven type selectors (named, inline, re-export chain, mixed, qualified
    // and `import type * as`) leave only their test-only members reported.
    let mut targets: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["target"].as_str().unwrap())
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        [
            "ChainTestOnly",
            "NamedTestOnly",
            "PresentType",
            "QualifiedTestOnly",
            "Square",
            "mixedTestOnlyValue",
        ],
        "{report}"
    );
    // A bare module type, an escaping namespace, a missing member and a missing
    // module stay unresolved instead of being dropped.
    let advisories = report["advisories"].as_array().unwrap();
    for (file, reason) in [
        ("other/whole-module.mts", "type import selector"),
        ("other/whole-namespace.mts", "namespace selector"),
        ("other/missing-member.mts", "unresolved symbol `Absent`"),
        ("other/unresolved-module.mts", "unresolved module selector"),
    ] {
        assert!(
            advisories.iter().any(|item| {
                item["file"]
                    .as_str()
                    .unwrap()
                    .replace('\\', "/")
                    .ends_with(file)
                    && item.to_string().contains(reason)
            }),
            "missing unresolved advisory for {file}: {report}"
        );
    }
    assert_eq!(advisories.len(), 4, "{report}");
    let root = crate::check_json_snapshot::rules_path(RULE, "fixture", "type-imports");
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_type_imports",
        &[("type_imports", root.clone(), root.join(".no-mistakes.yml"))],
    );
}

#[test]
fn type_references_do_not_consume_value_exports_snapshot() {
    let output = check("type-value-exports");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(output.status.code(), Some(1), "{report}");
    // Type-position references to functions, const, let and var never count as
    // production use (enums included: `Status` is reported, `TypeOnlyLevel` has
    // no runtime consumer at all); classes, interfaces and aliases keep counting.
    let mut targets: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["target"].as_str().unwrap())
        .collect();
    targets.sort_unstable();
    assert_eq!(
        targets,
        [
            "ConstStatus",
            "Status",
            "constInline",
            "fnChain",
            "fnImportQualified",
            "fnImportType",
            "fnNamespace",
            "fnQualified",
            "letValue",
            "starTestOnly",
            "testOnlyDefault",
            "varValue",
        ],
        "{report}"
    );
    // A default export's value-or-type kind is undetermined, so it is advised.
    let advisories = report["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 1, "{report}");
    assert!(
        advisories[0].to_string().contains("undeterminedDefault"),
        "{report}"
    );
    let root = crate::check_json_snapshot::rules_path(RULE, "fixture", "type-value-exports");
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_type_value_exports",
        &[(
            "type_value_exports",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}

#[test]
fn dynamic_imports_check_json_snapshot() {
    let output = check("dynamic-imports");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(output.status.success(), "{report}");
    // Proven dynamic selectors (`.then` member and destructure, a loader
    // returning the import, awaited member and destructure) are production use.
    assert!(report["rules"].as_array().unwrap().is_empty(), "{report}");
    // A computed member read, an unresolved module, a missing member, an unproven
    // import sharing a proven import's line, and a wrapper result that is
    // parenthesized, cast or stored before being chained keep their advisories.
    let advisories = report["advisories"].as_array().unwrap();
    let mut files: Vec<_> = advisories
        .iter()
        .map(|item| item["file"].as_str().unwrap().replace('\\', "/"))
        .collect();
    files.sort_unstable();
    assert_eq!(
        files,
        [
            "src/assigned-consumer.mts",
            "src/mixed-line.mts",
            "src/paren-consumer.mts",
            // `({ default: m.X, other })`, `({ ...m })` and `({ [k]: m.X })`.
            "src/then-default-extra-consumer.mts",
            "src/then-default-extra-consumer.mts",
            "src/then-default-extra-consumer.mts",
            "src/unknown-consumer.mts",
            "src/unresolved-consumer.mts",
            "src/unresolved-consumer.mts",
            "src/wrapper-consumer.mts",
            "src/wrapper-consumer.mts",
            // A whole-module `vi.mock` factory is a namespace use, not `default`.
            "tests/mock-factory.test.mts",
            "tests/mock-factory.test.mts",
        ],
        "{report}"
    );
    assert!(
        advisories
            .iter()
            .all(|item| !item["message"].as_str().unwrap().contains("`default`")),
        "{report}"
    );
    let root = crate::check_json_snapshot::rules_path(RULE, "fixture", "dynamic-imports");
    crate::check_json_snapshot::assert_saved_fixture(
        "production_exports_dynamic_imports",
        &[(
            "dynamic_imports",
            root.clone(),
            root.join(".no-mistakes.yml"),
        )],
    );
}
