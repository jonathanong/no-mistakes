use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/rules/{dir}"), config)
}

#[test]
fn csharp_max_lines_per_file_json_matches_snapshot() {
    let fail = "csharp-max-lines-per-file/fixture/fail";
    let config = |name: &str| format!("../../config/csharp_max_lines_per_file_{name}.yml");
    assert_saved_fixture(
        "csharp_max_lines_per_file",
        &[
            c("fail", fail, ".no-mistakes.yml"),
            c("under_limit", fail, &config("passes_under_limit")),
            c("test_max", fail, &config("uses_test_max")),
            c("over_limit", fail, &config("fails_over_limit")),
            c("disabled", fail, &config("disabled_skips")),
        ],
    );
}

#[test]
fn csharp_no_async_void_delegate_json_matches_snapshot() {
    let fail = "csharp-no-async-void-delegate/fixture/fail";
    assert_saved_fixture(
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
        ],
    );
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
        ],
    );
}
