use std::path::PathBuf;
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture_root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/import-usages/fixture"),
    )
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

#[test]
fn import_usages_subcommand_outputs_direct_import_rows() {
    let root = fixture_root();
    let output = Command::new(bin())
        .args([
            "import-usages",
            "src/main.mts",
            "--root",
            root.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("no-mistakes should run");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_str(&stdout(&output)).unwrap();
    let imports = json["files"][0]["imports"].as_array().unwrap();
    assert!(imports.iter().any(|row| {
        row["specifier"] == "@scope/pkg/register" && row["kind"] == "require-resolve"
    }));
}
