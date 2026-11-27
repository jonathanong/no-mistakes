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
