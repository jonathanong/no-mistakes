use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "no-cross-package-relative-imports";

#[test]
fn cross_package_relative_imports_match_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let reports = assert_saved_fixture_report(
        "cross_package_relative_imports",
        &[
            ("configured", root.clone(), root.join(".no-mistakes.yml")),
            ("default_off", root.clone(), root.join("default.yml")),
            ("incomplete", root.clone(), root.join("incomplete.yml")),
        ],
        &[],
        &["--include-suppressed"],
    );

    let configured = &reports["configured"];
    assert_eq!(configured["exitCode"], 1);
    assert!(configured["report"].get("incomplete").is_none());
    let findings = configured["report"]["rules"].as_array().unwrap();
    let mut locations: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            (
                finding["file"].as_str().unwrap(),
                finding["line"].as_u64().unwrap(),
            )
        })
        .collect();
    locations.sort_unstable();
    assert_eq!(
        locations,
        [
            ("packages/app/src/crossings.mts", 1),
            ("packages/app/src/crossings.mts", 2),
            ("packages/app/src/crossings.mts", 3),
            ("packages/app/src/crossings.mts", 4),
            ("packages/app/src/crossings.mts", 5),
            ("packages/app/src/crossings.mts", 9),
            ("packages/app/src/legacy.cjs", 1),
        ]
    );
    let advisories = configured["report"]["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 2);
    for line in [4, 7] {
        assert!(advisories.iter().any(|row| {
            row["file"] == "packages/app/src/accepted.mts"
                && row["line"] == line
                && row["message"].as_str().unwrap().starts_with("Unanalyzable")
        }));
    }
    assert_eq!(
        configured["report"]["suppressed"].as_array().unwrap().len(),
        3
    );

    let default = &reports["default_off"];
    assert_eq!(default["exitCode"], 0);
    assert_eq!(default["report"]["rules"], serde_json::json!([]));

    let incomplete = &reports["incomplete"];
    assert_eq!(incomplete["exitCode"], 1);
    let failures = incomplete["report"]["incomplete"].as_array().unwrap();
    assert!(failures
        .iter()
        .any(|row| row["consumer"] == RULE && row["file"] == "packages/app/src/gaps/broken.mts"));
    let findings = incomplete["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["file"], "packages/app/src/gaps/healthy.mts");
    assert_eq!(findings[0]["line"], 1);
}

#[test]
fn cross_package_relative_imports_reports_opaque_vitest_helpers() {
    let root = rules_path(RULE, "fixture", "");
    let cases = ["default", "report", "ignore"].map(|policy| {
        (
            policy,
            root.clone(),
            root.join(format!("helper-vitest-{policy}.yml")),
        )
    });
    let reports = assert_saved_fixture_report(
        "cross_package_relative_imports_vitest_helpers",
        &cases,
        &[],
        &[],
    );

    let advised = &reports["default"];
    assert_eq!(advised["exitCode"], 0);
    let findings = advised["report"]["advisories"].as_array().unwrap();
    let calls: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!(finding["rule"], RULE);
            assert!(finding.get("target").is_none());
            (
                finding["line"].as_u64().unwrap(),
                finding["import"].as_str().unwrap(),
            )
        })
        .collect();
    let crossing = "../../shared/src/status.mts";
    assert_eq!(
        calls,
        [
            (5, crossing),
            (6, crossing),
            (8, crossing),
            (9, "<computed>")
        ]
    );

    let reported = &reports["report"];
    assert_eq!(reported["exitCode"], 1);
    assert_eq!(reported["report"]["rules"], advised["report"]["advisories"]);
    assert_eq!(reported["report"]["advisories"], serde_json::json!([]));

    let ignored = &reports["ignore"];
    assert_eq!(ignored["exitCode"], 0);
    assert_eq!(ignored["report"]["rules"], serde_json::json!([]));
    assert_eq!(ignored["report"]["advisories"], serde_json::json!([]));
}

#[test]
fn cross_package_relative_imports_allow_pairs_match_snapshot() {
    let root = rules_path(RULE, "fixture", "");
    let cases = [
        ("allow_pairs", "allow.yml"),
        ("allow_target", "allow-target.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    let reports =
        assert_saved_fixture_report("cross_package_relative_imports_allow", &cases, &[], &[]);

    for label in ["allow_pairs", "allow_target"] {
        let report = &reports[label];
        assert_eq!(report["exitCode"], 1);
        let findings = report["report"]["rules"].as_array().unwrap();
        let expected_count = if label == "allow_pairs" { 3 } else { 4 };
        assert_eq!(findings.len(), expected_count, "{label}");
        let locations: Vec<_> = findings
            .iter()
            .map(|row| (row["file"].as_str().unwrap(), row["line"].as_u64().unwrap()))
            .collect();
        if label == "allow_pairs" {
            assert!(locations.contains(&("packages/app/nested/crossings.mts", 1)));
        } else {
            assert!(locations.contains(&("packages/app/nested/crossings.mts", 2)));
            assert!(locations.contains(&("packages/app/nested/crossings.mts", 3)));
        }
        assert!(locations.contains(&("packages/app/src/crossings.mts", 3)));
        assert!(locations.contains(&("packages/app/src/crossings.mts", 4)));
        let advisories = report["report"]["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), 2, "{label}");
        for line in [4, 7] {
            assert!(advisories.iter().any(|row| {
                row["file"] == "packages/app/src/accepted.mts"
                    && row["line"] == line
                    && row["message"].as_str().unwrap().starts_with("Unanalyzable")
            }));
        }
    }
}

#[test]
fn cross_package_relative_imports_rejects_invalid_allow_entries_before_reporting() {
    let root = rules_path(RULE, "fixture", "");
    let mut errors = serde_json::Map::new();
    for config in [
        "allow-empty-glob.yml",
        "allow-invalid-glob.yml",
        "allow-invalid-shape.yml",
        "allow-unknown-importer.yml",
        "allow-unknown-target.yml",
    ] {
        let stderr = ["1", "2"].map(|jobs| {
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["--lock-timeout", "0", "--jobs", jobs, "check", "--root"])
                .arg(&root)
                .args(["--config", config, "--format", "json"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2), "{config}");
            assert!(output.stdout.is_empty(), "{config}");
            String::from_utf8(output.stderr).unwrap()
        });
        assert_eq!(stderr[0], stderr[1], "{config}");
        errors.insert(config.to_owned(), serde_json::json!(stderr[0]));
    }
    insta::with_settings!({snapshot_path => "common/snapshots", prepend_module_to_snapshot => false, omit_expression => true}, {
        insta::assert_snapshot!(
            "cross_package_relative_imports_invalid_allow",
            format!("{}\n", serde_json::to_string_pretty(&errors).unwrap())
        );
    });
}

#[test]
fn cross_package_relative_imports_routes_unanalyzable_through_shared_policy() {
    let root = rules_path(RULE, "fixture", "");
    let cases = [
        ("computed_default", "computed-default.yml"),
        ("computed_repo_advise", "computed-repo-advise.yml"),
        ("computed_rule_advise", "computed-rule-advise.yml"),
        ("computed_repo_report", "computed-repo-report.yml"),
        ("computed_rule_report", "computed-rule-report.yml"),
        ("computed_rule_ignore", "computed-rule-ignore.yml"),
        ("allow_advise", "allow-advise.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    let reports = assert_saved_fixture_report(
        "cross_package_relative_imports_unanalyzable",
        &cases,
        &[],
        &[],
    );

    for label in [
        "computed_default",
        "computed_repo_advise",
        "computed_rule_advise",
    ] {
        let case = &reports[label];
        assert_eq!(case["exitCode"], 0, "{label}");
        assert_eq!(case["report"]["rules"], serde_json::json!([]), "{label}");
        let advisories = case["report"]["advisories"].as_array().unwrap();
        assert_eq!(advisories.len(), 1, "{label}");
        assert_eq!(advisories[0]["file"], "packages/app/computed/locale.mts");
        assert_eq!(advisories[0]["line"], 1);
    }

    for label in ["computed_repo_report", "computed_rule_report"] {
        let case = &reports[label];
        assert_eq!(case["exitCode"], 1, "{label}");
        let rules = case["report"]["rules"].as_array().unwrap();
        assert_eq!(rules.len(), 1, "{label}");
        assert_eq!(rules[0]["file"], "packages/app/computed/locale.mts");
        assert_eq!(rules[0]["line"], 1);
        assert_eq!(
            case["report"]["advisories"],
            serde_json::json!([]),
            "{label}"
        );
    }

    let ignore_case = &reports["computed_rule_ignore"];
    assert_eq!(ignore_case["exitCode"], 0);
    assert_eq!(ignore_case["report"]["rules"], serde_json::json!([]));
    assert_eq!(ignore_case["report"]["advisories"], serde_json::json!([]));

    let allow_advise = &reports["allow_advise"];
    assert_eq!(allow_advise["exitCode"], 1);
    assert_eq!(allow_advise["report"]["rules"].as_array().unwrap().len(), 3);
    let allow_advisories = allow_advise["report"]["advisories"].as_array().unwrap();
    assert_eq!(allow_advisories.len(), 2);
    for line in [4, 7] {
        assert!(allow_advisories.iter().any(|row| {
            row["file"] == "packages/app/src/accepted.mts"
                && row["line"] == line
                && row["message"].as_str().unwrap().starts_with("Unanalyzable")
        }));
    }
}

#[test]
fn cross_package_relative_imports_preserves_main_findings_for_same_package_vitest_and_assets() {
    let root = rules_path(RULE, "fixture", "");
    let cases = [
        ("compat_default", "compat-default.yml"),
        ("compat_report", "compat-report.yml"),
        ("compat_cross", "compat-cross.yml"),
        ("compat_cross_allow", "compat-cross-allow.yml"),
    ]
    .map(|(label, config)| (label, root.clone(), root.join(config)));
    let reports =
        assert_saved_fixture_report("cross_package_relative_imports_compat", &cases, &[], &[]);

    let default_case = &reports["compat_default"];
    assert_eq!(default_case["exitCode"], 0);
    assert_eq!(default_case["report"]["rules"], serde_json::json!([]));
    let default_advisories = default_case["report"]["advisories"].as_array().unwrap();
    let advisory_lines: Vec<_> = default_advisories
        .iter()
        .map(|row| row["line"].as_u64().unwrap())
        .collect();
    assert_eq!(advisory_lines, [17, 18]);

    let report_case = &reports["compat_report"];
    assert_eq!(report_case["exitCode"], 1);
    let report_lines: Vec<_> = report_case["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["line"].as_u64().unwrap())
        .collect();
    assert_eq!(report_lines, [17, 18]);
    assert_eq!(report_case["report"]["advisories"], serde_json::json!([]));

    let cross_case = &reports["compat_cross"];
    assert_eq!(cross_case["exitCode"], 1);
    assert_eq!(cross_case["report"]["advisories"], serde_json::json!([]));
    let mut cross_targets: Vec<_> = cross_case["report"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            (
                row["line"].as_u64().unwrap(),
                row["target"].as_str().unwrap(),
            )
        })
        .collect();
    cross_targets.sort_unstable();
    assert_eq!(
        cross_targets,
        [
            (3, "packages/shared/fixtures/tone.wav"),
            (8, "packages/shared/src/widget.tsx"),
            (11, "packages/shared/src/status.mts"),
        ]
    );

    let cross_allow = &reports["compat_cross_allow"];
    assert_eq!(cross_allow["exitCode"], 0);
    assert_eq!(cross_allow["report"]["rules"], serde_json::json!([]));
    assert_eq!(cross_allow["report"]["advisories"], serde_json::json!([]));
}
