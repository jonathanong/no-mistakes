use super::check_json_snapshot::assert_saved_fixture;
use std::path::PathBuf;

fn case(label: &'static str, name: &str, config: &str) -> (&'static str, PathBuf, PathBuf) {
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
            case(
                "unreadable_required_ts",
                "unreadable-required-ts",
                "both.yml",
            ),
            case("malformed_required_ts", "malformed-required-ts", "both.yml"),
            case(
                "malformed_required_json",
                "malformed-required-json",
                "both.yml",
            ),
        ],
    );
}

#[test]
fn clean_assets_exit_zero_and_required_sources_name_their_consumers() {
    for jobs in ["1", "2"] {
        for (name, expected_consumers) in [
            ("binary-only", &[][..]),
            ("directory", &[][..]),
            ("dangling-symlink", &[][..]),
            (
                "unreadable-required-ts",
                &["nextjs-no-caching", "production-dependency-declarations"][..],
            ),
            (
                "malformed-required-ts",
                &["nextjs-no-caching", "production-dependency-declarations"][..],
            ),
            (
                "malformed-required-json",
                &["production-dependency-declarations"][..],
            ),
        ] {
            let (_, root, config) = case("check", name, "both.yml");
            let output = std::process::Command::new(env!("CARGO_BIN_EXE_no-mistakes"))
                .args(["--jobs", jobs, "check", "--root"])
                .arg(root)
                .arg("--config")
                .arg(config)
                .args(["--format", "json"])
                .output()
                .unwrap();
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            let failures = report["incomplete"].as_array().cloned().unwrap_or_default();
            let consumers: Vec<_> = failures
                .iter()
                .map(|failure| failure["consumer"].as_str().unwrap())
                .collect();
            assert_eq!(consumers, expected_consumers, "{name}: {report}");
            assert!(
                failures
                    .iter()
                    .all(|failure| failure["file"].as_str().is_some_and(|file| file
                        .ends_with("page.ts")
                        || file.ends_with("package.json"))),
                "{name}: {report}"
            );
            assert_eq!(
                output.status.code(),
                Some(i32::from(!expected_consumers.is_empty())),
                "{name}: {report}"
            );
        }
    }
}
