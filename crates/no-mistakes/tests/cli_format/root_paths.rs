use super::*;

#[test]
fn global_check_relative_root_resolves() {
    let output = Command::new(bin())
        .current_dir(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        )
        .args(["check", "--root", "test-cases/queue-ast-hop/basic/fixture"])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
}

#[test]
fn react_analyze_relative_root_resolves() {
    let output = Command::new(bin())
        .current_dir(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap(),
        )
        .args([
            "react",
            "--root",
            "test-cases/react-traits-components/basic/fixture",
            "analyze",
            "app/components/Greeting.tsx",
        ])
        .output()
        .expect("no-mistakes should run");
    assert!(output.status.success());
}

#[test]
fn global_check_passes_on_react_fixture_with_no_violations() {
    let root = react_fixture("react-traits-components", "basic");
    let output = run(&["check", "--root", root.to_str().unwrap()]);
    assert!(output.status.success());
}
