use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn case(
    label: &'static str,
    name: &str,
    config: &str,
) -> (&'static str, PathBuf, PathBuf) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/check/partial-input-scope")
        .join(name);
    (label, root.clone(), root.join("../configs").join(config))
}

#[test]
fn unrelated_inputs_do_not_become_rule_prerequisites() {
    assert_saved_fixture(
        "partial_input_scope",
        &[
            case("binary_caching", "binary-only", "caching.yml"),
            case("binary_dependencies", "binary-only", "dependencies.yml"),
            case("binary_both", "binary-only", "both.yml"),
            case("healthy_both", "healthy", "both.yml"),
            case("directory", "directory", "both.yml"),
            case("dangling_symlink", "dangling-symlink", "both.yml"),
            case("unreadable_required_ts", "unreadable-required-ts", "both.yml"),
            case("malformed_required_ts", "malformed-required-ts", "both.yml"),
            case("malformed_required_json", "malformed-required-json", "both.yml"),
        ],
    );
}
