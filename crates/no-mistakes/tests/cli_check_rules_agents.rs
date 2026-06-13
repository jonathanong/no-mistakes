use std::path::PathBuf;
use std::process::{Command, Output};

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

fn check_fixture_config(root: &PathBuf, name: &str) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(name))
        .output()
        .unwrap()
}

fn check_fixture_format(root: &PathBuf, format: &str) -> Output {
    Command::new(bin())
        .args(["check", "--root"])
        .arg(root)
        .arg("--config")
        .arg(root.join(".no-mistakes.yml"))
        .args(["--format", format])
        .output()
        .unwrap()
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

#[test]
fn agents_md_max_size_advisory_does_not_fail_check() {
    let root = fixture("agents-md-max-size", "advisory");
    let out = check_fixture_config(&root, ".no-mistakes.yml");
    let body = stdout(&out);

    assert!(out.status.success(), "advisory must not fail check: {body}");
    assert!(body.contains("advisory agents-md-max-size"), "{body}");
    assert!(body.contains("CLAUDE.md"), "{body}");
    assert!(body.contains("12 characters / 12 bytes"), "{body}");
    assert!(body.contains("8 remaining"), "{body}");
}

#[test]
fn agents_md_max_size_json_output_includes_advisories() {
    let root = fixture("agents-md-max-size", "advisory");
    let out_json = check_fixture_format(&root, "json");
    let body = stdout(&out_json);

    assert!(
        out_json.status.success(),
        "advisory must not fail check: {body}"
    );
    assert!(body.contains("\"advisories\""), "{body}");
    assert!(body.contains("\"agents-md-max-size\""), "{body}");
    assert!(body.contains("8 remaining"), "{body}");
    assert!(body.contains("\"rules\": []"), "{body}");
}

#[test]
fn agents_md_max_size_paths_output_omits_advisories() {
    let root = fixture("agents-md-max-size", "advisory");
    let out = check_fixture_format(&root, "paths");
    let body = stdout(&out);

    assert!(out.status.success(), "advisory must not fail check: {body}");
    assert!(
        body.is_empty(),
        "paths output should omit advisories: {body}"
    );
}

#[test]
fn agents_md_max_size_md_output_includes_advisories() {
    let root = fixture("agents-md-max-size", "advisory");
    let out = check_fixture_format(&root, "md");
    let body = stdout(&out);

    assert!(out.status.success(), "advisory must not fail check: {body}");
    assert!(body.contains("## advisories"), "{body}");
    assert!(body.contains("agents-md-max-size"), "{body}");
}

#[test]
fn agents_md_max_size_yml_output_includes_advisories() {
    let root = fixture("agents-md-max-size", "advisory");
    let out = check_fixture_format(&root, "yml");
    let body = stdout(&out);

    assert!(out.status.success(), "advisory must not fail check: {body}");
    assert!(body.contains("advisories:"), "{body}");
    assert!(body.contains("agents-md-max-size"), "{body}");
}
