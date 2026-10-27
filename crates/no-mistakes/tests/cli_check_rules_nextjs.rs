use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn nextjs_caching_effort_low_preserves_literals_and_max_reports_at_least_as_much() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/no-nextjs-caching-effort/fixture");
    for (config, expected_findings, expected_advisories) in [
        (".no-mistakes.max.yml", 11, 0),
        (".no-mistakes.low.yml", 11, 0),
        (".no-mistakes.low-advise.yml", 7, 4),
        (".no-mistakes.low-ignore.yml", 7, 0),
    ] {
        let out = Command::new(bin())
            .args(["check", "--root"])
            .arg(&root)
            .arg("--config")
            .arg(root.join(config))
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
        let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            report["rules"].as_array().unwrap().len(),
            expected_findings,
            "{config}: {report}"
        );
        assert_eq!(
            super::policy_advisories::without_hygiene(&report).len(),
            expected_advisories,
            "{config}: {report}"
        );
        assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    }
}

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, scenario: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules")
            .join(category)
            .join("fixture")
            .join(scenario),
    )
}

fn config(rule: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules")
        .join(rule)
        .join("config")
        .join(name)
}

fn check(root: &Path, config: &Path) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(config)
        .output()
        .unwrap()
}

const REDIRECT: &str = "nextjs-redirect-destinations";
const REDIRECT_DEFAULT: &str = "nextjs_redirect_destinations_passes_for_existing_page.yml";
const REDIRECT_TUPLES: &str =
    "nextjs_static_tuple_destinations_and_partial_extraction_are_checked.yml";
const REDIRECT_EXPLICIT_MAX: &str =
    "nextjs_redirect_effort_max_matches_default_and_low_keeps_literals-explicit-max.yml";
const REDIRECT_LOW_REPORT: &str =
    "nextjs_redirect_effort_max_matches_default_and_low_keeps_literals-low-report.yml";
const REDIRECT_LOW_ADVISE: &str =
    "nextjs_redirect_effort_max_matches_default_and_low_keeps_literals-low-advise.yml";
const REDIRECT_LOW_IGNORE: &str =
    "nextjs_redirect_effort_max_matches_default_and_low_keeps_literals-low-ignore.yml";
const REDIRECT_LOW: &str =
    "nextjs_redirect_effort_max_matches_default_and_low_keeps_literals-low.yml";

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn nextjs_redirect_destinations_passes_for_existing_page() {
    let root = fixture("nextjs-redirect-destinations", "pass");
    let out = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    assert!(out.status.success(), "exit non-zero: {}", stdout(&out));
}

#[test]
fn nextjs_redirect_destinations_fails_for_missing_page() {
    let root = fixture("nextjs-redirect-destinations", "fail-missing");
    let out = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("/gone"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_flags_private_underscore_routes() {
    let root = fixture("nextjs-redirect-destinations", "skip-private");
    let out = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("/secret"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_checks_rewrites_by_default() {
    let root = fixture("nextjs-redirect-destinations", "rewrite-fail");
    let out = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(body.contains("rewrite destination"), "{body}");
}

#[test]
fn nextjs_redirect_destinations_flags_extractor_drift() {
    let root = fixture("nextjs-redirect-destinations", "fail-extractor");
    let out = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    let body = stdout(&out);

    assert!(!out.status.success(), "expected exit 1");
    assert!(body.contains("nextjs-redirect-destinations"), "{body}");
    assert!(
        body.contains("could not locate the redirects() body"),
        "{body}"
    );
}

#[test]
fn nextjs_static_tuple_destinations_and_partial_extraction_are_checked() {
    for (scenario, expected) in [
        ("static-tuples-fail", "/missing-route"),
        ("partial-dynamic", "extraction is incomplete"),
    ] {
        let root = fixture("nextjs-redirect-destinations", scenario);
        let out = check(&root, &config(REDIRECT, REDIRECT_TUPLES));
        assert_eq!(out.status.code(), Some(1));
        assert!(stdout(&out).contains(expected), "{}", stdout(&out));
    }
    let root = fixture("nextjs-redirect-destinations", "static-tuples-pass");
    assert!(check(&root, &config(REDIRECT, REDIRECT_TUPLES))
        .status
        .success());
}

#[test]
fn nextjs_redirect_effort_max_matches_default_and_low_keeps_literals() {
    let root = fixture("nextjs-redirect-destinations", "dest-non-string");
    let default = check(&root, &config(REDIRECT, REDIRECT_DEFAULT));
    let explicit_max = check(&root, &config(REDIRECT, REDIRECT_EXPLICIT_MAX));
    assert!(default.status.success());
    assert_eq!(stdout(&default), stdout(&explicit_max));
    let low_report = check(&root, &config(REDIRECT, REDIRECT_LOW_REPORT));
    assert_eq!(low_report.status.code(), Some(1));
    assert!(
        stdout(&low_report).contains("effort low"),
        "{}",
        stdout(&low_report)
    );
    assert!(
        stdout(&low_report).contains("literal array of objects"),
        "{}",
        stdout(&low_report)
    );
    let advise = check(&root, &config(REDIRECT, REDIRECT_LOW_ADVISE));
    assert!(advise.status.success(), "{}", stdout(&advise));
    assert!(
        stdout(&advise).contains("effort low"),
        "{}",
        stdout(&advise)
    );
    let ignore = check(&root, &config(REDIRECT, REDIRECT_LOW_IGNORE));
    assert!(ignore.status.success(), "{}", stdout(&ignore));
    assert!(!stdout(&ignore).contains("nextjs-redirect-destinations"));
    let pass = fixture("nextjs-redirect-destinations", "pass");
    assert!(check(&pass, &config(REDIRECT, REDIRECT_LOW))
        .status
        .success());
    let missing = fixture("nextjs-redirect-destinations", "fail-missing");
    let missing_low = check(&missing, &config(REDIRECT, REDIRECT_LOW));
    assert_eq!(missing_low.status.code(), Some(1));
    assert!(
        stdout(&missing_low).contains("/gone"),
        "{}",
        stdout(&missing_low)
    );
}

#[test]
fn nextjs_redirect_low_mixed_literals_stay_checked_under_unanalyzable() {
    let root = fixture("nextjs-redirect-destinations", "mixed-literal-unanalyzable");
    for (yaml, saved) in [
        (".no-mistakes-low-report.yml", REDIRECT_LOW_REPORT),
        (".no-mistakes-low-advise.yml", REDIRECT_LOW_ADVISE),
        (".no-mistakes-low-ignore.yml", REDIRECT_LOW_IGNORE),
    ] {
        let out = check(&root, &config(REDIRECT, saved));
        let body = stdout(&out);
        assert_eq!(out.status.code(), Some(1), "{yaml}: {body}");
        for destination in [
            "/missing-literal",
            "/missing-tpl",
            "/missing-rw-literal",
            "/missing-rw-tpl",
        ] {
            assert!(body.contains(destination), "{yaml} {destination}: {body}");
        }
    }
}
