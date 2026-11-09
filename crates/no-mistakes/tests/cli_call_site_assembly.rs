use std::path::PathBuf;

#[test]
fn call_site_assembly_check_json_snapshot() {
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let small = repo.join("test-cases/codebase-analysis/call-site-assembly/fixture");
    let large = repo.join("test-cases/performance/call-site-assembly");
    let cases = [
        ("small-on", small.clone(), small.join("on.yml")),
        ("small-off", small.clone(), small.join("off.yml")),
        ("large-on", large.clone(), large.join("on.yml")),
        ("large-off", large.clone(), large.join("off.yml")),
        ("large-imports", large.clone(), large.join("imports.yml")),
    ];
    crate::check_json_snapshot::assert_saved_fixture_with_env_and_args(
        "call_site_assembly",
        &cases,
        &[],
        &["--timeout", "120"],
    );
}
