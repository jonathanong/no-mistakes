use super::common::{assert_success, fixture, run, run_in, run_json, stdout};

#[test]
fn rsc_callers_json_reports_server_callers() {
    let root = fixture("rsc-callers");
    let value = run_json(&root, &["rsc-callers", "app/ui/Button.tsx"]);
    assert_eq!(value["component"], "app/ui/Button.tsx");

    let files: Vec<&str> = value["callers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["file"].as_str().unwrap())
        .collect();
    assert!(files.contains(&"app/ui/Card.tsx"));
    assert!(files.contains(&"app/ui/ServerWidget.tsx"));
    assert!(files.contains(&"app/dashboard/page.tsx"));
    // Client boundary and everything above it are excluded.
    assert!(!files.contains(&"app/ui/ClientThing.tsx"));
    assert!(!files.contains(&"app/ui/ClientParent.tsx"));
}

#[test]
fn rsc_callers_malformed_tsconfig_errors() {
    let root = fixture("rsc-callers");
    let root_arg = root.to_string_lossy();
    let output = run(&[
        "rsc-callers",
        "app/ui/Button.tsx",
        "--root",
        root_arg.as_ref(),
        "--tsconfig",
        "bad-tsconfig.json",
    ]);
    assert!(!output.status.success());
}

#[test]
fn rsc_callers_human_and_md_formats() {
    let root = fixture("rsc-callers");
    let human = run_in(&root, &["rsc-callers", "app/ui/Button.tsx"]);
    assert_success(&human);
    assert!(stdout(&human).contains("app/ui/Card.tsx"));

    let md = run_in(
        &root,
        &["rsc-callers", "app/ui/Button.tsx", "--format", "md"],
    );
    assert_success(&md);
    assert!(stdout(&md).contains("# RSC callers of"));

    let paths = run_in(
        &root,
        &["rsc-callers", "app/ui/Button.tsx", "--format", "paths"],
    );
    assert_success(&paths);
    assert!(stdout(&paths).contains("app/dashboard/page.tsx"));

    let yml = run_in(
        &root,
        &["rsc-callers", "app/ui/Button.tsx", "--format", "yml"],
    );
    assert_success(&yml);
    assert!(stdout(&yml).contains("component:"));
}
