use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn csp_allowed_origins_json_matches_snapshot() {
    let report = rules_path("csp-allowed-origins", "fixture", "report");
    let headers_text = rules_path("csp-allowed-origins", "fixture", "headers-text");
    let browser_semantics = rules_path("csp-allowed-origins", "fixture", "browser-semantics");
    let unanalyzable = rules_path("csp-allowed-origins", "fixture", "unanalyzable-and-urls");
    let yaml_block = rules_path("csp-allowed-origins", "fixture", "yaml-block");
    let dynamic_policies = rules_path("csp-allowed-origins", "fixture", "dynamic-policies");
    let mixed_static_dynamic = rules_path("csp-allowed-origins", "fixture", "mixed-static-dynamic");
    let parse_error_target = rules_path("csp-allowed-origins", "fixture", "parse-error-target");
    let const_registries = rules_path("csp-allowed-origins", "fixture", "const-registries");
    assert_saved_fixture(
        "csp_allowed_origins",
        &[
            ("report", report.clone(), report.join(".no-mistakes.yml")),
            (
                "default-off",
                report.clone(),
                report.join("default-off.yml"),
            ),
            (
                "headers-text",
                headers_text.clone(),
                headers_text.join(".no-mistakes.yml"),
            ),
            (
                "headers-meta-mixed-quotes",
                headers_text.clone(),
                headers_text.join("meta-mixed-quotes.yml"),
            ),
            (
                "duplicate-directive",
                browser_semantics.clone(),
                browser_semantics.join("duplicate.yml"),
            ),
            (
                "multi-policy-strings",
                browser_semantics.clone(),
                browser_semantics.join("multi-string.yml"),
            ),
            (
                "multi-policy-files",
                browser_semantics.clone(),
                browser_semantics.join("multi-file.yml"),
            ),
            (
                "unanalyzable-report",
                unanalyzable.clone(),
                unanalyzable.join("report.yml"),
            ),
            (
                "unanalyzable-advise",
                unanalyzable.clone(),
                unanalyzable.join("advise.yml"),
            ),
            (
                "unanalyzable-ignore",
                unanalyzable.clone(),
                unanalyzable.join("ignore.yml"),
            ),
            (
                "yaml-block",
                yaml_block.clone(),
                yaml_block.join(".no-mistakes.yml"),
            ),
            (
                "dynamic-nonce-join",
                dynamic_policies.clone(),
                dynamic_policies.join("nonce-join.yml"),
            ),
            (
                "dynamic-join-report",
                dynamic_policies.clone(),
                dynamic_policies.join("dynamic-join-report.yml"),
            ),
            (
                "dynamic-join-advise",
                dynamic_policies.clone(),
                dynamic_policies.join("dynamic-join-advise.yml"),
            ),
            (
                "dynamic-ternary",
                dynamic_policies.clone(),
                dynamic_policies.join("ternary.yml"),
            ),
            (
                "mixed-static-dynamic-report",
                mixed_static_dynamic.clone(),
                mixed_static_dynamic.join("report.yml"),
            ),
            (
                "mixed-static-dynamic-advise",
                mixed_static_dynamic.clone(),
                mixed_static_dynamic.join("advise.yml"),
            ),
            (
                "mixed-static-dynamic-ignore",
                mixed_static_dynamic.clone(),
                mixed_static_dynamic.join("ignore.yml"),
            ),
            (
                "parse-error-target-report",
                parse_error_target.clone(),
                parse_error_target.join("report.yml"),
            ),
            (
                "parse-error-target-advise",
                parse_error_target.clone(),
                parse_error_target.join("advise.yml"),
            ),
            (
                "parse-error-target-ignore",
                parse_error_target.clone(),
                parse_error_target.join("ignore.yml"),
            ),
            (
                "const-registry-same-module",
                const_registries.clone(),
                const_registries.join("same-module.yml"),
            ),
            (
                "const-registry-imported",
                const_registries.clone(),
                const_registries.join("imported.yml"),
            ),
            (
                "const-registry-mutated",
                const_registries.clone(),
                const_registries.join("mutated.yml"),
            ),
        ],
    );
}

#[test]
fn csp_allowed_origins_invalid_options_fail_early_with_exit_2() {
    let root = rules_path("csp-allowed-origins", "fixture", "errors");
    for (config, expected) in [
        ("unknown-option.yml", "bogusOption"),
        ("empty-csp-files.yml", "requires non-empty options.cspFiles"),
        ("blank-csp-file.yml", "must be nonempty paths"),
        ("invalid-self-origin.yml", "must be a valid origin"),
    ] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
            .args(["--lock-timeout", "0", "--jobs", "1", "check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json"])
            .output()
            .unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert_eq!(output.status.code(), Some(2), "{config}: {stderr}");
        assert!(output.stdout.is_empty(), "{config}: {stderr}");
        assert!(stderr.contains(expected), "{config}: {stderr}");
    }
}
