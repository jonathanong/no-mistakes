use super::check_json_case::case;
use super::check_json_snapshot::{assert_saved_fixture, assert_saved_fixture_report};

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/rules/{dir}"), config)
}

#[test]
fn csharp_max_lines_per_file_json_matches_snapshot() {
    let fail = "csharp-max-lines-per-file/fixture/fail";
    let config = |name: &str| format!("../../config/csharp_max_lines_per_file_{name}.yml");
    let input = "file-input-analysis/fixture";
    let reports = assert_saved_fixture_report(
        "csharp_max_lines_per_file",
        &[
            c("fail", fail, ".no-mistakes.yml"),
            c("under_limit", fail, &config("passes_under_limit")),
            c("test_max", fail, &config("uses_test_max")),
            c("over_limit", fail, &config("fails_over_limit")),
            c("disabled", fail, &config("disabled_skips")),
            c(
                "disable_line",
                "csharp-max-lines-per-file/fixture/suppression",
                "../fail/.no-mistakes.yml",
            ),
            // The file-length finding is anchored at line 1.
            c(
                "first_line_next_line_directive",
                "csharp-max-lines-per-file/fixture/suppression-next-line",
                "../fail/.no-mistakes.yml",
            ),
            c("input_report", input, "csharp-max-lines-per-file.yml"),
            c(
                "input_advise",
                input,
                "csharp-max-lines-per-file-advise.yml",
            ),
            c(
                "input_ignore",
                input,
                "csharp-max-lines-per-file-ignore.yml",
            ),
        ],
        &[],
        &[],
    );
    let suppressed = &reports["disable_line"]["report"]["rules"];
    let findings = suppressed.as_array().unwrap();
    assert_eq!(findings.len(), 1, "{suppressed}");
    assert_eq!(findings[0]["file"], "Control.cs");
    assert_eq!(findings[0]["line"], 1);

    for (label, expected_rules, expected_advisories) in [
        ("input_report", 2, 0),
        ("input_advise", 1, 1),
        ("input_ignore", 1, 0),
    ] {
        let result = &reports[label];
        assert_eq!(result["exitCode"], 1);
        assert!(result["report"].get("incomplete").is_none(), "{result}");
        let rules = result["report"]["rules"].as_array().unwrap();
        let advisories = result["report"]["advisories"].as_array().unwrap();
        assert_eq!(rules.len(), expected_rules, "{result}");
        assert_eq!(advisories.len(), expected_advisories, "{result}");
        assert!(rules
            .iter()
            .any(|finding| finding["file"] == "csharp/definite.cs"));
        assert_eq!(
            rules
                .iter()
                .any(|finding| finding["file"] == "csharp/bad.cs"
                    && finding["rule"] == "csharp-max-lines-per-file"
                    && finding["target"] == "unanalyzable"),
            label == "input_report"
        );
        assert_eq!(
            advisories
                .iter()
                .any(|finding| finding["file"] == "csharp/bad.cs"
                    && finding["rule"] == "csharp-max-lines-per-file"
                    && finding["target"] == "unanalyzable"),
            label == "input_advise"
        );
    }
}

#[test]
fn csharp_no_async_void_delegate_json_matches_snapshot() {
    let fail = "csharp-no-async-void-delegate/fixture/fail";
    let reports = assert_saved_fixture_report(
        "csharp_no_async_void_delegate",
        &[
            c("fail", fail, ".no-mistakes.yml"),
            c(
                "cli_config",
                fail,
                "../../config/csharp_no_async_void_delegate_cli_fails.yml",
            ),
            c(
                "pass",
                "csharp-no-async-void-delegate/fixture/pass",
                ".no-mistakes.yml",
            ),
            c(
                "disable_lines",
                "csharp-no-async-void-delegate/fixture/suppression",
                "../fail/.no-mistakes.yml",
            ),
        ],
        &[],
        &[],
    );
    let result = &reports["disable_lines"];
    assert_eq!(result["exitCode"], 1);
    let findings = result["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 1, "{result}");
    assert_eq!(findings[0]["file"], "PageBinding.cs");
    assert_eq!(findings[0]["line"], 11);
}

#[test]
fn swift_no_raw_print_json_matches_snapshot() {
    let fail = "swift-no-raw-print/fixture/fail";
    assert_saved_fixture(
        "swift_no_raw_print",
        &[
            c("fail", fail, ".no-mistakes.yml"),
            c(
                "cli_config",
                fail,
                "../../config/swift_no_raw_print_cli_fails.yml",
            ),
            c(
                "pass",
                "swift-no-raw-print/fixture/pass",
                ".no-mistakes.yml",
            ),
            c(
                "disable_lines",
                "swift-no-raw-print/fixture/disable-lines",
                "../../config/swift_no_raw_print_cli_fails.yml",
            ),
            c(
                "disable_file",
                "swift-no-raw-print/fixture/disable-file",
                "../../config/swift_no_raw_print_cli_fails.yml",
            ),
        ],
    );
}

#[test]
fn invalid_utf8_swift_and_csharp_json_matches_snapshot() {
    assert_saved_fixture(
        "invalid_utf8_masked_source",
        &[
            c(
                "swift_print",
                "swift-no-raw-print/fixture/invalid-utf8",
                ".no-mistakes.yml",
            ),
            c(
                "swift_viewmodel",
                "swift-viewmodel-main-actor/fixture/invalid-utf8",
                ".no-mistakes.yml",
            ),
            c(
                "csharp_async_void",
                "csharp-no-async-void-delegate/fixture/invalid-utf8",
                ".no-mistakes.yml",
            ),
        ],
    );
}

#[test]
fn swift_viewmodel_main_actor_json_matches_snapshot() {
    let fail = "swift-viewmodel-main-actor/fixture/fail";
    assert_saved_fixture(
        "swift_viewmodel_main_actor",
        &[
            c("fail", fail, ".no-mistakes.yml"),
            c(
                "cli_config",
                fail,
                "../../config/swift_viewmodel_main_actor_cli_fails.yml",
            ),
            c(
                "pass",
                "swift-viewmodel-main-actor/fixture/pass",
                ".no-mistakes.yml",
            ),
            c(
                "disable_lines",
                "swift-viewmodel-main-actor/fixture/disable-lines",
                "../../config/swift_viewmodel_main_actor_cli_fails.yml",
            ),
            c(
                "disable_file",
                "swift-viewmodel-main-actor/fixture/disable-file",
                "../../config/swift_viewmodel_main_actor_cli_fails.yml",
            ),
        ],
    );
}

#[test]
fn swift_no_unguarded_darwin_import_json_matches_snapshot() {
    use super::check_json_snapshot::assert_saved_fixture_report;

    let fixture = "swift-no-unguarded-darwin-import/fixture";
    let reports = assert_saved_fixture_report(
        "swift_no_unguarded_darwin_import",
        &[
            c("configured", fixture, "../configs/check.yml"),
            c("default_off", fixture, "../configs/default-off.yml"),
            c("allow_and_message", fixture, "../configs/allow.yml"),
            c("advise", fixture, "../configs/advise.yml"),
            c("ignore", fixture, "../configs/ignore.yml"),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["configured"]["exitCode"], 1);
    assert_eq!(
        reports["configured"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        15
    );
    assert_eq!(
        reports["advise"]["report"]["advisories"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    let ignore_advisories = reports["ignore"]["report"]["advisories"]
        .as_array()
        .unwrap();
    assert_eq!(ignore_advisories.len(), 1);
    assert_eq!(ignore_advisories[0]["rule"], "unused-disable-directive");
    assert_eq!(
        reports["ignore"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        9
    );
    assert_eq!(reports["default_off"]["exitCode"], 0);
    assert_eq!(
        reports["default_off"]["report"]["rules"],
        serde_json::json!([])
    );
}

#[test]
fn swift_apple_module_imports_json_matches_snapshot() {
    use super::check_json_snapshot::assert_saved_fixture_report;

    let fixture = "swift-no-unguarded-darwin-import/apple-modules";
    let reports = assert_saved_fixture_report(
        "swift_apple_module_imports",
        &[
            c("default_modules", fixture, "../configs/check.yml"),
            c(
                "opt_in_apple_modules",
                fixture,
                "../configs/apple-opt-in.yml",
            ),
            c("custom_modules", fixture, "../configs/modules.yml"),
            c("empty_modules", fixture, "../configs/no-modules.yml"),
            c(
                "macos_app_source_skipped",
                "swift-no-unguarded-darwin-import/scope-macos-app",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "xcodegen_app_skipped",
                "swift-no-unguarded-darwin-import/scope-xcodegen-app",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "apple_platforms_package_portable_test",
                "swift-no-unguarded-darwin-import/scope-apple-platforms",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "apple_guard_conjunctions",
                "swift-no-unguarded-darwin-import/scope-apple-guards",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "portable_security_default",
                "swift-no-unguarded-darwin-import/scope-portable-security",
                "../configs/repo-default.yml",
            ),
            c(
                "portable_security_opt_in",
                "swift-no-unguarded-darwin-import/scope-portable-security",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "dynamic_platforms_unanalyzable",
                "swift-no-unguarded-darwin-import/scope-dynamic-manifest",
                "../configs/apple-opt-in.yml",
            ),
            c(
                "dynamic_platforms_advise",
                "swift-no-unguarded-darwin-import/scope-dynamic-manifest",
                "../configs/apple-opt-in-advise.yml",
            ),
            c(
                "dynamic_targets_unanalyzable",
                "swift-no-unguarded-darwin-import/scope-dynamic-targets",
                "../configs/repo-default.yml",
            ),
        ],
        &[],
        &[],
    );
    assert_eq!(
        reports["default_modules"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        reports["opt_in_apple_modules"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        10
    );
    assert_eq!(
        reports["custom_modules"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    for clean in [
        "empty_modules",
        "macos_app_source_skipped",
        "xcodegen_app_skipped",
        "apple_guard_conjunctions",
        "portable_security_default",
    ] {
        assert_eq!(reports[clean]["exitCode"], 0, "{clean}");
        assert_eq!(
            reports[clean]["report"]["rules"],
            serde_json::json!([]),
            "{clean}"
        );
    }
    assert_eq!(
        reports["apple_platforms_package_portable_test"]["exitCode"],
        1
    );
    assert_eq!(
        reports["portable_security_opt_in"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        reports["dynamic_platforms_unanalyzable"]["report"]["rules"][0]["target"],
        "unanalyzable"
    );
    assert_eq!(
        reports["dynamic_platforms_advise"]["report"]["advisories"][0]["target"],
        "unanalyzable"
    );
    assert_eq!(
        reports["dynamic_targets_unanalyzable"]["report"]["rules"][0]["target"],
        "unanalyzable"
    );
}

#[test]
fn swift_apple_module_imports_reject_invalid_module_options_before_emitting_a_report() {
    let (_, root, config) = c(
        "invalid_module",
        "swift-no-unguarded-darwin-import/apple-modules",
        "../configs/invalid-modules.yml",
    );
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
        .args(["--lock-timeout", "0", "check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let error = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{error}");
    assert!(output.stdout.is_empty());
    assert!(error.contains("options.modules[1]"), "{error}");
}
