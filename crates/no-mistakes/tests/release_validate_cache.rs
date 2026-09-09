use std::path::PathBuf;

fn release_workflow() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/release.yml");
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()))
}

#[test]
fn validate_job_drops_cargo_package_tree_before_rust_cache_save() {
    let workflow = release_workflow();
    let publish = workflow
        .find("cargo publish --dry-run --locked -p no-mistakes")
        .expect("Validate must dry-run cargo publish");
    let cleanup_heading = workflow
        .find("name: Drop cargo package verification tree")
        .expect("Validate must drop the cargo package tree before rust-cache save");
    let cleanup = workflow[cleanup_heading..]
        .find("rm -rf target/package")
        .map(|offset| cleanup_heading + offset)
        .expect(
            "cleanup must remove target/package so rust-cache does not opendir missing tests/target and tests/trybuild paths",
        );
    assert!(
        publish < cleanup_heading && cleanup_heading < cleanup,
        "package verification tree must be removed after cargo publish --dry-run"
    );
    let step = &workflow[cleanup_heading..cleanup];
    assert!(
        step.contains("if: always()"),
        "cleanup must run even when cargo publish fails after creating the tree"
    );
}
