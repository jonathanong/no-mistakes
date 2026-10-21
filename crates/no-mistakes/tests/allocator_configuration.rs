//! Compile the real registration independently of the analyzer and native allocator.
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

fn registration() -> String {
    let library = include_str!("../src/lib.rs");
    library
        .split_once("pub mod ast;")
        .expect("library registration boundary moved")
        .0
        .to_owned()
}

fn compile_configuration(prefix: &str, testing: bool, feature: bool) -> Output {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = std::fs::read_to_string(
        root.join("fixtures/performance/mimalloc/allocator-configuration.rs"),
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let mut command = Command::new("rustc");
    command.args([
        "--edition=2021",
        "--crate-name",
        "allocator_configuration",
        "--emit=metadata",
        "-o",
    ]);
    command.arg(output.path().join("test.rmeta")).arg("-");
    if testing {
        command.arg("--test");
    }
    if feature {
        command.args(["--cfg", "feature=\"mimalloc\""]);
    }
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{prefix}{fixture}").as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn registration_allows_unit_test_instrumentation_and_production() {
    for testing in [false, true] {
        for feature in [false, true] {
            let result = compile_configuration(&registration(), testing, feature);
            assert!(
                result.status.success(),
                "testing={testing}, feature={feature}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}

#[test]
fn collision_control_rejects_two_global_allocators() {
    // Removing not(test) reproduces the collision with allocation-count guards.
    let original = registration().replace(
        "all(feature = \"mimalloc\", not(test))",
        "feature = \"mimalloc\"",
    );
    let result = compile_configuration(&original, true, true);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("cannot define multiple global allocators"));
}
