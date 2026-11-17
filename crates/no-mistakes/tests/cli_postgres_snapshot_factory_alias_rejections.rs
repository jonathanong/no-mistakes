use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

#[test]
fn postgres_factory_alias_rejections_check_json_snapshot() {
    let rule = "postgres-lock-written-rows-only";
    let root = rules_path(rule, "fixture", "factory-alias-rejections");
    let reports = assert_saved_fixture_report(
        "postgres_factory_alias_rejections",
        &[
            (
                "no_options",
                root.clone(),
                rules_path(rule, "configs", "checks-fail.yml"),
            ),
            (
                "with_options",
                root,
                rules_path(rule, "configs", "factory-alias-options.yml"),
            ),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["no_options"], reports["with_options"]);
    let report = &reports["no_options"];
    assert_eq!(report["exitCode"], 1);
    let findings = report["report"]["rules"].as_array().unwrap();
    for (file, line) in [
        ("pool-reassigned.ts", 6),
        ("pool.ts", 4),
        ("pool-member.ts", 6),
        ("conditional.ts", 5),
        ("post-commit.ts", 6),
        ("post-disposal.ts", 8),
        ("destructured.ts", 6),
        ("property.ts", 6),
        ("pre-transaction.ts", 5),
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding["file"] == format!("src/{file}")
                    && finding["line"] == line
                    && finding["target"] == "unanalyzable"
            }),
            "missing rejection for {file}:{line}: {report}"
        );
    }
    assert_eq!(findings.len(), 9, "{report}");
    assert!(!findings.iter().any(|finding| matches!(
        finding["file"].as_str(),
        Some("src/accepted.ts" | "src/ordinary.ts" | "src/shadowed.ts")
    )));
}
