use serde_json::Value;

#[test]
fn node_loader_origins_remain_visible_without_binding_resolution() {
    let root =
        crate::check_json_snapshot::rules_path("test-no-live-repository-scan", "node-loaders", "");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["check", "--format", "json", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let advisories = report["advisories"].as_array().unwrap();
    let expected = [
        (
            "module.test.ts",
            vec!["module-literal", "module-computed", "module-bracket"],
        ),
        (
            "factory.test.ts",
            vec![
                "direct-factory",
                "computed-factory",
                "namespace-factory",
                "default-factory",
                "bound-loader",
                "loader-alias",
                "factory-alias",
                "mutated-loader",
            ],
        ),
        (
            "commonjs.test.cjs",
            vec![
                "commonjs-named-factory",
                "commonjs-namespace-factory",
                "commonjs-direct-factory",
            ],
        ),
        (
            "mutations.test.ts",
            vec![
                "escaped-loader",
                "mutated-namespace",
                "mutated-factory",
                "mutated-module",
            ],
        ),
        (
            "long-alias.test.ts",
            vec!["long-loader-alias", "original-loader-after-long-alias"],
        ),
        (
            "parity.test.ts",
            vec!["existing-scanner-after-reassignment"],
        ),
        ("arguments.test.ts", vec!["spread-module-argument"]),
    ];
    for (name, markers) in expected {
        let file = format!("tests/{name}");
        let source = std::fs::read_to_string(root.join(&file)).unwrap();
        for marker in markers {
            let line = source
                .lines()
                .position(|line| line.ends_with(&format!("// {marker}")))
                .unwrap()
                + 1;
            assert!(
                advisories
                    .iter()
                    .any(|finding| finding["file"] == file && finding["line"] == line),
                "missing {marker}: {report}"
            );
        }
    }
    // The Node fallback must not replace the canonical unresolved scan finding.
    assert!(
        advisories
            .iter()
            .any(|finding| finding["file"] == "tests/parity.test.ts"
                && finding["line"] == 5
                && finding["import"] == "readdirSync"),
        "{report}"
    );
    assert_eq!(advisories.len(), 23, "{report}");
    let findings = report["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{report}");
    assert_eq!(findings[0]["file"], "tests/segments.test.ts");
    assert_eq!(findings[0]["line"], 2);
    crate::check_json_snapshot::assert_saved_fixture(
        "test_no_live_repository_scan_node_loaders",
        &[
            ("default", root.clone(), root.join(".no-mistakes.yml")),
            ("report", root.clone(), root.join("report.yml")),
            ("custom", root.clone(), root.join("custom.yml")),
            ("advisory", root.clone(), root.join("advisory.yml")),
        ],
    );
}

#[test]
fn node_loader_fallback_preserves_policy_filters_messages_and_suppression() {
    let root =
        crate::check_json_snapshot::rules_path("test-no-live-repository-scan", "node-loaders", "");
    let run = |config: &str, suppressed: bool| {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command
            .args(["check", "--format", "json", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config));
        if suppressed {
            command.arg("--include-suppressed");
        }
        let output = command.output().unwrap();
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    };
    let report = run("report.yml", false);
    assert_eq!(report["rules"].as_array().unwrap().len(), 24, "{report}");
    assert!(report["advisories"].as_array().unwrap().is_empty());
    let advisory = run("advisory.yml", false);
    assert!(advisory["rules"].as_array().unwrap().is_empty());
    assert_eq!(advisory["advisories"].as_array().unwrap().len(), 24);
    let custom = run("custom.yml", false);
    let advisories = custom["advisories"].as_array().unwrap();
    assert_eq!(advisories.len(), 2, "{custom}");
    assert!(advisories
        .iter()
        .all(|finding| finding["message"] == "Use a declared fixture directory."));
    let suppressed = run(".no-mistakes.yml", true);
    assert_eq!(
        suppressed["suppressed"].as_array().unwrap().len(),
        3,
        "{suppressed}"
    );
}
