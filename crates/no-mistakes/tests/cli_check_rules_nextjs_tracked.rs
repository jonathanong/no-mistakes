#[allow(dead_code)]
#[path = "common/gitignore_fixture.rs"]
mod gitignore_fixture;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn git(root: &Path, args: &[&str]) {
    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_COMMON_DIR")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_WORK_TREE")
        .status()
        .unwrap()
        .success());
}

fn check(root: &Path, config: &str, node: bool) -> Output {
    let bin = PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"));
    let mut command = if node {
        let mut command = Command::new("node");
        // Use the actual Node launcher with this test's compiled native binary.
        command.args(["-e", "const {launchNative}=require(process.argv[1]); launchNative(process.argv.slice(3),undefined,process,undefined,()=>({cliPath:process.argv[2]}));"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packages/no-mistakes/bin/no-mistakes.js"))
            .arg(&bin);
        command
    } else {
        Command::new(bin)
    };
    command
        .args(["check", "--root"])
        .arg(root)
        .args(["--format", "json", "--config"])
        .arg(root.join(config))
        .output()
        .unwrap()
}

#[test]
fn native_and_node_cli_require_tracked_routes_when_enabled() {
    let fixture = gitignore_fixture::materialize_saved(
        "../../test-cases/rules/nextjs-redirect-destinations/fixture/tracked-routes",
    );
    let root = fixture.path();
    std::fs::rename(root.join(".gitignore.fixture"), root.join(".gitignore")).unwrap();
    git(root, &["init", "-q", "--initial-branch=main"]);
    git(
        root,
        &[
            "add",
            ".gitignore",
            ".no-mistakes.yml",
            ".filesystem.yml",
            "next.config.ts",
            "app/tracked",
            "app/(group)",
            "app/posts",
            "app/docs",
            "app/optional",
        ],
    );
    let mut baseline = None;
    for node in [false, true] {
        let output = check(root, ".no-mistakes.yml", node);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), 6);
        if let Some(baseline) = &baseline {
            assert_eq!(&output.stdout, baseline);
        } else {
            baseline = Some(output.stdout);
        }
        let default = check(root, ".filesystem.yml", node);
        let report: serde_json::Value = serde_json::from_slice(&default.stdout).unwrap();
        assert_eq!(report["rules"].as_array().unwrap().len(), 3);
    }
    git(root, &["add", "-f", "app/untracked", "app/ignored"]);
    for node in [false, true] {
        let output = check(root, ".no-mistakes.yml", node);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
