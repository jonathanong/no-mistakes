use super::common::{assert_success, fixture, run_in, run_json, stdout};

#[test]
fn registry_extension_json_register_call() {
    let root = fixture("registry-extension");
    let value = run_json(&root, &["registry-extension", "register-call.ts"]);
    assert_eq!(value["patternKind"], "register-call");
    assert_eq!(value["registrant"], "registry.register");
    assert_eq!(value["entries"].as_array().unwrap().len(), 2);
}

#[test]
fn registry_extension_json_container() {
    let root = fixture("registry-extension");
    let value = run_json(&root, &["registry-extension", "container-array.ts"]);
    assert_eq!(value["patternKind"], "container-array");
}

#[test]
fn registry_extension_human_and_md_formats() {
    let root = fixture("registry-extension");
    let human = run_in(&root, &["registry-extension", "register-call.ts"]);
    assert_success(&human);
    assert!(stdout(&human).contains("register-call"));

    let md = run_in(
        &root,
        &["registry-extension", "register-call.ts", "--format", "md"],
    );
    assert_success(&md);
    assert!(stdout(&md).contains("# registry-extension"));

    let paths = run_in(
        &root,
        &[
            "registry-extension",
            "register-call.ts",
            "--format",
            "paths",
        ],
    );
    assert_success(&paths);
    assert!(stdout(&paths).contains("register-call.ts"));
    assert!(stdout(&paths).contains("./plugins/foo"));
}

#[test]
fn registry_extension_yml_and_notes() {
    let root = fixture("registry-extension");
    let yml = run_in(
        &root,
        &["registry-extension", "register-call.ts", "--format", "yml"],
    );
    assert_success(&yml);
    assert!(stdout(&yml).contains("registryFile:") || stdout(&yml).contains("patternKind:"));

    // mixed.ts has a side-effect import note; exercise the human notes loop.
    let human = run_in(&root, &["registry-extension", "mixed.ts"]);
    assert_success(&human);
    assert!(stdout(&human).contains("note:"));
}

#[test]
fn registry_extension_none() {
    let root = fixture("registry-extension");
    let value = run_json(&root, &["registry-extension", "none.ts"]);
    assert_eq!(value["patternKind"], "none");
    assert!(value["entries"].as_array().unwrap().is_empty());
}
