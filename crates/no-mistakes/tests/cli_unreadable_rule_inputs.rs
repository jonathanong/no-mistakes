use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn unreadable_rule_inputs_json_matches_snapshot() {
    let rule = "required-doc-section";
    let fixture =
        super::saved_fixture::materialize_path(&rules_path(rule, "fixture", "unreadable-inputs"));
    let root = fixture.path();
    for args in [
        &["init", "--quiet"][..],
        &["add", "--force", "--", "docs", "tests"],
    ] {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for (link, path) in [
        ("unavailable-doc.link", "docs/unavailable.md"),
        ("unavailable-test.link", "tests/unavailable.test.ts"),
    ] {
        std::fs::rename(root.join(link), root.join(path)).unwrap();
    }

    let cases = ["report", "advise", "ignore"].map(|mode| {
        (
            mode,
            root.to_path_buf(),
            rules_path(rule, "configs", &format!("unreadable-{mode}.yml")),
        )
    });
    assert_saved_fixture("unreadable_rule_inputs", &cases);
}
