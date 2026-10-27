use super::check_json_case::case;
use super::check_json_snapshot::assert_saved_fixture;

type Case = (&'static str, std::path::PathBuf, std::path::PathBuf);

fn c(label: &'static str, dir: &str, config: &str) -> Case {
    case(label, &format!("test-cases/{dir}"), config)
}

fn redirect(label: &'static str, scenario: &str, config: &str) -> Case {
    let dir = format!("rules/nextjs-redirect-destinations/fixture/{scenario}");
    c(label, &dir, config)
}

#[test]
fn nextjs_no_api_routes_json_matches_snapshot() {
    assert_saved_fixture(
        "nextjs_no_api_routes",
        &[c(
            "fail",
            "codebase-analysis/no-nextjs-api-routes/fixture",
            "../../../rules/nextjs-no-api-routes/config/nextjs_no_api_routes_fails_for_app_and_pages_routes.yml",
        )],
    );
}

#[test]
fn nextjs_no_caching_json_matches_snapshot() {
    assert_saved_fixture(
        "nextjs_no_caching",
        &[c(
            "fail",
            "codebase-analysis/no-nextjs-caching/fixture",
            "../../../rules/nextjs-no-caching/config/nextjs_no_caching_fails_for_cache_features.yml",
        )],
    );
}

#[test]
fn nextjs_redirect_destinations_json_matches_snapshot() {
    let max = ".no-mistakes.yml";
    assert_saved_fixture(
        "nextjs_redirect_destinations",
        &[
            redirect("pass", "pass", max),
            redirect("missing", "fail-missing", max),
            redirect("missing_low", "fail-missing", ".no-mistakes-low.yml"),
            redirect("rewrite_fail", "rewrite-fail", max),
            redirect("extractor", "fail-extractor", max),
            redirect("reexport", "fail-reexport", max),
            redirect("skip_private", "skip-private", max),
            redirect("tuples_fail", "static-tuples-fail", max),
            redirect("tuples_pass", "static-tuples-pass", max),
            redirect("disable_file", "disable-file", max),
            redirect("dynamic", "dynamic", max),
            redirect("non_string", "dest-non-string", max),
            redirect(
                "non_string_advise",
                "dest-non-string",
                ".no-mistakes-low-advise.yml",
            ),
            redirect(
                "non_string_ignore",
                "dest-non-string",
                ".no-mistakes-low-ignore.yml",
            ),
            redirect("mixed", "mixed-literal-unanalyzable", max),
            redirect(
                "mixed_low_report",
                "mixed-literal-unanalyzable",
                ".no-mistakes-low-report.yml",
            ),
            redirect(
                "alias_low_advise",
                "low-path-alias",
                ".no-mistakes-low-advise.yml",
            ),
        ],
    );
}

#[test]
fn server_route_client_boundary_json_matches_snapshot() {
    let fail = "rules/server-route-client-boundary/fixture/fail";
    assert_saved_fixture(
        "server_route_client_boundary",
        &[
            c("fail", fail, "check-config.yml"),
            c("boundary_only", fail, "boundary-only.no-mistakes.yml"),
            c("plain_config", fail, "no-mistakes.yml"),
            c(
                "pass",
                "rules/server-route-client-boundary/fixture/pass",
                "check-config.yml",
            ),
            c(
                "non_source_files",
                "rules/server-route-client-boundary/fixture/non-source-project-files",
                ".no-mistakes.yml",
            ),
        ],
    );
}
