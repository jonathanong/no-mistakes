use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(category: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases")
            .join(category)
            .join(name)
            .join("fixture"),
    )
}

fn repository_fixture(category: &str, name: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures")
            .join(category)
            .join(name),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf8")
}

#[test]
fn global_check_timings_are_reported_in_phase_order() {
    let root = fixture("codebase-analysis", "check-configured-only");
    let output = run(&[
        "check",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--timings",
    ]);

    assert!(output.status.success());
    let err = stderr(&output);
    let mut previous = 0;
    for label in ["react:", "queues:", "rules:", "integration:", "codebase:"] {
        let index = err
            .find(label)
            .unwrap_or_else(|| panic!("missing {label} in {err}"));
        assert!(index >= previous, "{label} should be in phase order: {err}");
        previous = index;
    }
}

#[test]
fn global_check_verbose_timings_reports_fine_grained_labels() {
    let root = fixture("codebase-analysis", "forbidden-dependencies-basic");
    let output = run(&[
        "check",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--timings",
        "--verbose-timings",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let err = stderr(&output);

    // Ensure core timing labels are present, including Playwright-related labels.
    for label in [
        "rules.forbidden_dependencies:",
        "graph.imports:",
        "playwright.",
    ] {
        assert!(
            err.contains(label),
            "expected verbose timing label {label} in stderr: {err}"
        );
    }

    // Ensure the `[timing]` prefix formatting is present so downstream tools relying on it
    // will break visibly if the format changes.
    let has_timing_prefix = err
        .lines()
        .any(|line| line.starts_with("[timing] ") || line.contains(" [timing] "));
    assert!(
        has_timing_prefix,
        "expected at least one line with a `[timing]` prefix in stderr: {err}"
    );
    assert_eq!(err.matches("[timing] total:").count(), 1, "{err}");
    for label in [
        "rules.forbidden_dependencies:",
        "graph.imports:",
        "playwright.",
    ] {
        let line = err
            .lines()
            .find(|line| line.contains(label))
            .unwrap_or_else(|| panic!("missing {label} in {err}"));
        assert!(
            line.ends_with("(parallel; non-additive)"),
            "nested parallel timing must be marked non-additive: {line}"
        );
    }
    assert!(err.contains("[work] output.renders: 1"), "{err}");
}

fn without_diagnostics(stderr: &[u8]) -> Vec<u8> {
    String::from_utf8_lossy(stderr)
        .lines()
        .filter(|line| !line.starts_with("[timing] ") && !line.starts_with("[work] "))
        .flat_map(|line| [line.as_bytes(), b"\n"].concat())
        .collect()
}

#[test]
fn diagnostics_preserve_public_output_status_and_errors() {
    let finding_root = repository_fixture("diagnostics", "public-output-parity");
    for (format, expected_name) in [
        ("json", "check.json"),
        ("yml", "check.yml"),
        ("md", "check.md"),
        ("paths", "check.paths"),
        ("human", "check.txt"),
    ] {
        let base = [
            "check",
            "--root",
            finding_root.to_str().unwrap(),
            "--format",
            format,
        ];
        let plain = run(&base);
        let expected = std::fs::read(finding_root.join("expected").join(expected_name))
            .expect("frozen public output fixture should be readable");
        let mut timed_args = base.to_vec();
        timed_args.push("--verbose-timings");
        let timed = run(&timed_args);
        assert_eq!(plain.status.code(), Some(1), "{format}");
        assert_eq!(plain.stdout, expected, "frozen {format} public output");
        assert_eq!(timed.status.code(), plain.status.code(), "{format}");
        assert_eq!(timed.stdout, plain.stdout, "{format}");
        assert_eq!(without_diagnostics(&timed.stderr), plain.stderr, "{format}");
    }

    let success_root = fixture("codebase-analysis", "check-configured-only");
    let plain = run(&["check", "--root", success_root.to_str().unwrap(), "--json"]);
    let timed = run(&[
        "check",
        "--root",
        success_root.to_str().unwrap(),
        "--json",
        "--timings",
    ]);
    assert_eq!(timed.status.code(), plain.status.code());
    assert_eq!(timed.stdout, plain.stdout);
    assert_eq!(without_diagnostics(&timed.stderr), plain.stderr);

    let invalid_root = fixture("react-traits-config", "invalid");
    let plain = run(&["check", "--root", invalid_root.to_str().unwrap(), "--json"]);
    let timed = run(&[
        "check",
        "--root",
        invalid_root.to_str().unwrap(),
        "--json",
        "--verbose-timings",
    ]);
    assert_eq!(plain.status.code(), Some(2));
    assert_eq!(timed.status.code(), plain.status.code());
    assert_eq!(timed.stdout, plain.stdout);
    assert_eq!(without_diagnostics(&timed.stderr), plain.stderr);
    assert!(stderr(&timed).contains("[work] output.errors: 1"));
    assert!(!stderr(&timed).contains("[work] output.renders:"));
}

#[test]
fn global_check_without_verbose_timings_omits_fine_grained_labels() {
    let root = fixture("codebase-analysis", "forbidden-dependencies-basic");
    let output = run(&[
        "check",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--timings",
    ]);

    assert_eq!(output.status.code(), Some(1));
    let err = stderr(&output);
    assert!(
        !err.contains("rules.forbidden_dependencies:") && !err.contains("graph.imports:"),
        "verbose timing labels should not appear without --verbose-timings: {err}"
    );
    assert!(
        !err.contains("[work]"),
        "basic timings must omit work metrics: {err}"
    );
}

/// Regression test for the `collect_app_selector_occurrences` duplicate-work
/// fix: when both a Playwright rule and `forbidden-dependencies` are
/// configured together, `rules.playwright` builds its own Playwright
/// analysis and `rules.forbidden_dependencies`'s `DepGraph` build
/// (`graph.playwright_selectors`) builds another — both used to pay the full
/// app-wide selector scan independently. This only proves both real call
/// sites run in one invocation without erroring; the actual "only computed
/// once" guarantee is proven deterministically (via a call-count assertion)
/// by `get_or_compute_app_selector_occurrences_caches_per_scan_html_ids_key`
/// in the graph module's own tests — timing-based assertions here would be
/// flaky under CI load.
#[test]
fn global_check_shares_app_selector_scan_between_playwright_rule_and_forbidden_dependencies() {
    let root = fixture(
        "codebase-analysis",
        "playwright-coverage-and-forbidden-dependencies",
    );
    let output = run(&[
        "check",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "--verbose-timings",
    ]);

    let err = stderr(&output);
    for label in ["rules.playwright:", "graph.playwright_selectors:"] {
        assert!(
            err.contains(label),
            "expected both the playwright rule and the forbidden-dependencies graph build to run: {err}"
        );
    }
}
