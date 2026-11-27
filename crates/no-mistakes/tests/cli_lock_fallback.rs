use crate::saved_fixture;
use serde_json::Value;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct ReadOnlyRuntime(PathBuf);

impl ReadOnlyRuntime {
    fn new(root: &Path) -> Self {
        let runtime = root.join("runtime/no-mistakes");
        set_mode(&runtime.join("invocation.lock"), 0o444);
        set_mode(&runtime, 0o555);
        Self(runtime)
    }
}

impl Drop for ReadOnlyRuntime {
    fn drop(&mut self) {
        set_mode(&self.0, 0o700);
    }
}

fn set_mode(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}

fn isolated_command(mut command: Command, root: &Path) -> Command {
    command
        .env_remove("CARGO_BIN_EXE_no-mistakes")
        .env_remove("NO_MISTAKES_LOCK_DIR")
        .env("HOME", root)
        .env("XDG_RUNTIME_DIR", root.join("runtime"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("TMPDIR", root.join("temp"));
    command
}

fn cli(root: &Path) -> Command {
    isolated_command(Command::new(env!("CARGO_BIN_EXE_no-mistakes")), root)
}

fn success_json(output: &Output) -> Value {
    assert!(output.status.success(), "{output:?}");
    serde_json::from_slice(&output.stdout).unwrap()
}

fn fallback_warning(output: &Output) -> Value {
    let warnings: Vec<Value> = String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|item| item["kind"] == "invocation_lock_fallback")
        .collect();
    assert_eq!(warnings.len(), 1, "{output:?}");
    warnings.into_iter().next().unwrap()
}

#[test]
fn read_only_runtime_keeps_check_and_plan_reports() {
    let fixture = saved_fixture::materialize("invocation", "read-only-runtime");
    let root = fixture.path();
    let analyze = |args: &[&str]| {
        cli(root)
            .args(args)
            .args(["--root", root.join("repo").to_str().unwrap()])
            .args(["--format", "json", "--jobs", "1"])
            .output()
            .unwrap()
    };
    let check = success_json(&analyze(&["check"]));
    let plan = success_json(&analyze(&[
        "tests",
        "plan",
        "vitest",
        "--changed-file",
        "message.ts",
    ]));

    let runtime = ReadOnlyRuntime::new(root);
    let denied = runtime.0.join("invocation.lock");
    assert_eq!(
        std::fs::OpenOptions::new()
            .write(true)
            .open(&denied)
            .unwrap_err()
            .kind(),
        io::ErrorKind::PermissionDenied
    );
    for (args, baseline) in [
        (&["check"][..], check),
        (
            &["tests", "plan", "vitest", "--changed-file", "message.ts"][..],
            plan,
        ),
    ] {
        let output = analyze(args);
        assert_eq!(success_json(&output), baseline);
        let warning = fallback_warning(&output);
        assert_eq!(warning["failed"][0]["path"], denied.to_str().unwrap());
        assert_eq!(
            warning["lockFile"],
            root.join("cache/no-mistakes/invocation.lock")
                .to_str()
                .unwrap()
        );
    }
    let status = cli(root)
        .args(["lock", "status", "--format", "json"])
        .output()
        .unwrap();
    assert_eq!(
        success_json(&status)["lockFile"],
        root.join("cache/no-mistakes/invocation.lock")
            .to_str()
            .unwrap()
    );
}

#[test]
fn override_takes_precedence_and_unusable_cache_falls_back_to_private_temp() {
    let fixture = saved_fixture::materialize("invocation", "read-only-runtime");
    let root = fixture.path();
    let _runtime = ReadOnlyRuntime::new(root);
    let status = |command: &mut Command| {
        command
            .args(["lock", "status", "--format", "json"])
            .output()
            .unwrap()
    };
    let override_dir = root.join("override");
    let explicit = status(cli(root).env("NO_MISTAKES_LOCK_DIR", &override_dir));
    assert_eq!(
        success_json(&explicit)["lockFile"],
        override_dir.join("invocation.lock").to_str().unwrap()
    );
    assert!(explicit.stderr.is_empty(), "{explicit:?}");

    set_mode(&root.join("temp"), 0o700);
    let blocked = root.join("repo/message.ts");
    let fallback = status(cli(root).env("XDG_CACHE_HOME", &blocked));
    let warning = fallback_warning(&fallback);
    assert_eq!(warning["failed"].as_array().unwrap().len(), 2);
    let report = success_json(&fallback);
    assert_eq!(report["state"], "free");
    assert_eq!(report["lockFile"], warning["lockFile"]);
    let path = Path::new(report["lockFile"].as_str().unwrap());
    assert!(path.starts_with(root.join("temp")), "{path:?}");
    assert_eq!(
        path.parent()
            .unwrap()
            .metadata()
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o700
    );
}

#[test]
fn unavailable_isolated_lock_keeps_analysis_and_reports_no_path() {
    let fixture = saved_fixture::materialize("invocation", "read-only-runtime");
    let root = fixture.path();
    let command = || {
        let mut command = cli(root);
        command
            .env("CARGO_BIN_EXE_no-mistakes", "1")
            .env("TMPDIR", root.join("repo/message.ts"));
        command
    };
    let analyzed = command()
        .args([
            "check",
            "--root",
            root.join("repo").to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(success_json(&analyzed)["rules"].is_array());
    assert!(fallback_warning(&analyzed)["lockFile"].is_null());
    let status = command()
        .args(["lock", "status", "--format", "json"])
        .output()
        .unwrap();
    let result = success_json(&status);
    assert_eq!(result["state"], "unavailable");
    assert!(result["lockFile"].is_null());
    assert!(result["holder"].is_null());
    assert!(fallback_warning(&status)["lockFile"].is_null());
}

#[path = "cli_lock_fallback/concurrency.rs"]
mod concurrency;
