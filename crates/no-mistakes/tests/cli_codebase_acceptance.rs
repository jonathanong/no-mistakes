use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_no-mistakes"))
}

fn fixture(name: &str) -> PathBuf {
    no_mistakes_core::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase-analysis")
            .join(name),
    )
}

fn run(args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn run_in(root: &Path, args: &[&str]) -> Output {
    Command::new(bin())
        .current_dir(root)
        .args(args)
        .output()
        .expect("no-mistakes should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf8")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "exit code: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_json(root: &Path, args: &[&str]) -> Value {
    let mut command_args = Vec::from(args);
    command_args.extend(["--root", root.to_str().unwrap(), "--format", "json"]);
    let output = run(&command_args);
    assert_success(&output);
    serde_json::from_str(&stdout(&output)).unwrap_or_else(|e| {
        panic!("invalid JSON: {e}\nstdout: {}", stdout(&output));
    })
}

fn file_paths(value: &Value) -> Vec<String> {
    value["files"]
        .as_array()
        .map(|files| {
            files
                .iter()
                .filter_map(|file| file["path"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn via_kinds(value: &Value, path: &str) -> Vec<String> {
    value["files"]
        .as_array()
        .and_then(|files| {
            files
                .iter()
                .find(|file| file["path"].as_str() == Some(path))
        })
        .and_then(|file| file["via"].as_array())
        .map(|kinds| {
            kinds
                .iter()
                .filter_map(|kind| kind.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn has_path_with_via(value: &Value, path: &str, via: &str) -> bool {
    value["files"].as_array().into_iter().flatten().any(|file| {
        file["path"].as_str() == Some(path)
            && file["via"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some(via)))
    })
}

fn has_queue_job_with_via(value: &Value, queue_file: &str, job: &str, via: &str) -> bool {
    value["files"].as_array().into_iter().flatten().any(|file| {
        file["queueFile"].as_str() == Some(queue_file)
            && file["job"].as_str() == Some(job)
            && file["via"]
                .as_array()
                .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some(via)))
    })
}

#[test]
fn dependencies_acceptance_basic_cli_behaviors() {
    let root = fixture("simple");

    let output = run_in(
        &root,
        &[
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "a.mts",
        ],
    );
    assert_success(&output);
    assert_eq!(
        file_paths(&serde_json::from_str(&stdout(&output)).unwrap()),
        vec!["b.mts", "c.mts"]
    );

    for args in [
        vec!["dependencies", "--root", root.to_str().unwrap(), "a.mts"],
        vec![
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--json",
            "a.mts",
        ],
    ] {
        let output = run_in(&root, &args);
        assert_success(&output);
        assert!(serde_json::from_str::<Value>(&stdout(&output)).unwrap()["files"].is_array());
    }

    let yml = run_in(
        &root,
        &[
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "yml",
            "a.mts",
        ],
    );
    assert_success(&yml);
    assert!(stdout(&yml).contains("files:"));

    let human = run_in(
        &root,
        &[
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "human",
            "a.mts",
        ],
    );
    assert_success(&human);
    assert!(stdout(&human).contains("a.mts") && stdout(&human).contains("b.mts"));

    let multiple = run_json(&root, &["dependencies", "a.mts", "b.mts"]);
    assert!(file_paths(&multiple).contains(&"c.mts".to_string()));

    let depth = run_json(&root, &["dependencies", "--depth", "1", "a.mts"]);
    for file in depth["files"].as_array().expect("files array") {
        assert!(file["depth"].as_u64().unwrap() <= 1);
    }

    let paths = run_in(
        &root,
        &[
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "paths",
            "a.mts",
        ],
    );
    assert_success(&paths);
    assert!(stdout(&paths).lines().any(|line| line == "b.mts"));

    let missing = run_in(
        &root,
        &[
            "dependencies",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "nonexistent_xyz.mts",
        ],
    );
    assert_success(&missing);
    assert!(
        serde_json::from_str::<Value>(&stdout(&missing)).unwrap()["files"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let relative_root = run(&[
        "dependencies",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "json",
        "a.mts",
    ]);
    assert_success(&relative_root);
    assert!(!file_paths(&serde_json::from_str(&stdout(&relative_root)).unwrap()).is_empty());

    let invalid = run(&[
        "dependencies",
        "--root",
        root.to_str().unwrap(),
        "--relationship",
        "typo",
        "a.mts",
    ]);
    assert!(!invalid.status.success());

    let serial = run_json(&root, &["-j", "1", "dependencies", "a.mts"]);
    let parallel = run_json(&root, &["-j", "8", "dependencies", "a.mts"]);
    assert_eq!(parallel, serial);
}

#[test]
fn dependents_acceptance_basic_cli_behaviors() {
    let root = fixture("dependents-basic");

    for args in [
        vec![
            "dependents",
            "--root",
            root.to_str().unwrap(),
            "--relationship",
            "import",
            "src/source.mts",
        ],
        vec![
            "dependents",
            "--root",
            root.to_str().unwrap(),
            "--json",
            "--relationship",
            "import",
            "src/source.mts",
        ],
    ] {
        let output = run(&args);
        assert_success(&output);
        assert!(serde_json::from_str::<Value>(&stdout(&output)).unwrap()["files"].is_array());
    }

    let yml = run(&[
        "dependents",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "yml",
        "--relationship",
        "import",
        "src/source.mts",
    ]);
    assert_success(&yml);
    assert!(stdout(&yml).contains("files:"));

    let human = run(&[
        "dependents",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "human",
        "--relationship",
        "import",
        "src/source.mts",
    ]);
    assert_success(&human);
    assert!(stdout(&human).contains("src/source.mts") && stdout(&human).contains("src/mid.mts"));

    let multiple = run_json(
        &root,
        &["dependents", "src/source.mts", "scripts/child.mts"],
    );
    let multiple_paths = file_paths(&multiple);
    assert!(multiple_paths.contains(&"src/mid.mts".to_string()));
    assert!(multiple_paths.contains(&"scripts/runner.mts".to_string()));

    let depth = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "import",
            "--depth",
            "1",
            "src/source.mts",
        ],
    );
    let depth_paths = file_paths(&depth);
    assert!(depth_paths.contains(&"src/mid.mts".to_string()));
    assert!(!depth_paths.contains(&"src/top.mts".to_string()));

    let paths = run(&[
        "dependents",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "paths",
        "--relationship",
        "import",
        "src/source.mts",
    ]);
    assert_success(&paths);
    assert_eq!(
        stdout(&paths).lines().collect::<Vec<_>>(),
        vec!["src/mid.mts", "src/top.mts"]
    );

    let missing = run_in(
        &root,
        &[
            "dependents",
            "--root",
            root.to_str().unwrap(),
            "--format",
            "json",
            "nonexistent_xyz_97.mts",
        ],
    );
    assert_success(&missing);
    assert!(
        serde_json::from_str::<Value>(&stdout(&missing)).unwrap()["files"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    let relative = run_json(
        &root,
        &["dependents", "--relationship", "import", "src/source.mts"],
    );
    assert!(file_paths(&relative).contains(&"src/mid.mts".to_string()));

    let md = run_json(
        &root,
        &["dependents", "--relationship", "md", "src/source.mts"],
    );
    assert_eq!(file_paths(&md), vec!["docs/source-link.md"]);
    assert_eq!(via_kinds(&md, "docs/source-link.md"), vec!["md"]);

    let test = run_json(
        &root,
        &["dependents", "--relationship", "test", "src/source.mts"],
    );
    assert_eq!(file_paths(&test), vec!["src/source.test.mts"]);
    assert_eq!(via_kinds(&test, "src/source.test.mts"), vec!["test"]);

    let process = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "process",
            "scripts/child.mts",
        ],
    );
    assert_eq!(file_paths(&process), vec!["scripts/runner.mts"]);
    assert_eq!(via_kinds(&process, "scripts/runner.mts"), vec!["process"]);
}

#[test]
fn symbols_acceptance_cli_behaviors() {
    let root = fixture("simple");

    let help = run(&["symbols", "--help"]);
    assert_success(&help);
    let help = stdout(&help);
    assert!(help.contains("FILE"));
    assert!(help.contains("--include"));
    assert!(help.contains("--kind"));

    let missing_file_arg = run(&["symbols"]);
    assert!(!missing_file_arg.status.success());

    let default_json = run(&["symbols", "--root", root.to_str().unwrap(), "a.mts"]);
    assert_success(&default_json);
    let value: Value = serde_json::from_str(&stdout(&default_json)).unwrap();
    assert_eq!(value["roots"][0], "a.mts");
    assert_eq!(value["files"][0]["exports"][0]["name"], "a");
    assert!(value["files"][0].get("imports").is_none());

    let include_both = run(&[
        "symbols",
        "--root",
        root.to_str().unwrap(),
        "--include",
        "both",
        "--format",
        "json",
        "a.mts",
    ]);
    assert_success(&include_both);
    let value: Value = serde_json::from_str(&stdout(&include_both)).unwrap();
    let import = &value["files"][0]["imports"][0];
    assert_eq!(import["imported"], "b");
    assert_eq!(import["source"], "./b.mts");
    assert_eq!(import["resolved"], "b.mts");

    let invalid_kind = run(&[
        "symbols",
        "--root",
        root.to_str().unwrap(),
        "--kind",
        "functoin",
        "a.mts",
    ]);
    assert!(!invalid_kind.status.success());

    let paths = run(&[
        "symbols",
        "--root",
        root.to_str().unwrap(),
        "--format",
        "paths",
        "a.mts",
    ]);
    assert_success(&paths);
    let lines = stdout(&paths)
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 1);
    assert!(lines[0].starts_with("a.mts:"));
    assert!(lines[0].ends_with(":a"));
}

#[test]
fn cross_boundary_workspace_and_symbol_contracts() {
    let root = fixture("cross-boundary-monorepo");
    let backend_tsconfig = root.join("apps/backend/tsconfig.json");
    let web_tsconfig = root.join("apps/web/tsconfig.json");
    let root_tsconfig = root.join("tsconfig.json");

    let workspace_deps = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "--relationship",
            "workspace",
            "apps/backend/api/handler.mts",
        ],
    );
    assert!(file_paths(&workspace_deps).contains(&"packages/core/src/index.mts".to_string()));

    let workspace_reverse = run_json(
        &root,
        &[
            "dependents",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "--relationship",
            "workspace",
            "packages/core/src/index.mts",
        ],
    );
    assert!(file_paths(&workspace_reverse).contains(&"apps/backend/api/handler.mts".to_string()));

    let subpath = run_json(
        &root,
        &[
            "dependents",
            "--tsconfig",
            web_tsconfig.to_str().unwrap(),
            "--relationship",
            "workspace",
            "packages/core/src/types.mts",
        ],
    );
    assert!(file_paths(&subpath).contains(&"apps/web/pages/subpath.tsx".to_string()));

    let alias_deps = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "--relationship",
            "import",
            "apps/backend/api/handler.mts",
        ],
    );
    assert!(file_paths(&alias_deps).contains(&"apps/backend/services/topics/get.mts".to_string()));

    let alias_reverse = run_json(
        &root,
        &[
            "dependents",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "--relationship",
            "import",
            "apps/backend/services/topics/get.mts",
        ],
    );
    assert!(file_paths(&alias_reverse).contains(&"apps/backend/api/handler.mts".to_string()));

    let web_alias_deps = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            web_tsconfig.to_str().unwrap(),
            "--relationship",
            "import",
            "apps/web/pages/index.tsx",
        ],
    );
    assert!(file_paths(&web_alias_deps).contains(&"packages/core/src/types.mts".to_string()));

    let web_alias_reverse = run_json(
        &root,
        &[
            "dependents",
            "--tsconfig",
            web_tsconfig.to_str().unwrap(),
            "--relationship",
            "import",
            "packages/core/src/types.mts",
        ],
    );
    assert!(file_paths(&web_alias_reverse).contains(&"apps/web/pages/index.tsx".to_string()));

    let full_deps = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "apps/backend/api/handler.mts",
        ],
    );
    let types_path = "apps/backend/services/topics/types.mts";
    assert!(file_paths(&full_deps).contains(&types_path.to_string()));
    assert!(via_kinds(&full_deps, types_path).contains(&"type-import".to_string()));
    assert!(!via_kinds(&full_deps, types_path).contains(&"import".to_string()));

    let symbols = run_json(&root, &["symbols", "packages/core/src/index.mts"]);
    let exports = symbols["files"][0]["exports"].as_array().unwrap();
    let internal_helper = exports
        .iter()
        .find(|export| export["name"].as_str() == Some("internalHelper"))
        .unwrap();
    assert_eq!(internal_helper["kind"], "re-export");

    let symbol_dependents = run_json(
        &root,
        &[
            "dependents",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "packages/core/src/internal.mts#internalHelper",
        ],
    );
    assert!(file_paths(&symbol_dependents).contains(&"apps/backend/api/handler.mts".to_string()));

    let serial = run_json(
        &root,
        &[
            "-j",
            "1",
            "dependents",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "packages/core/src/internal.mts#internalHelper",
        ],
    );
    let parallel = run_json(
        &root,
        &[
            "-j",
            "8",
            "dependents",
            "--tsconfig",
            backend_tsconfig.to_str().unwrap(),
            "packages/core/src/internal.mts#internalHelper",
        ],
    );
    assert_eq!(parallel, serial);

    let extends = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            root_tsconfig.to_str().unwrap(),
            "apps/backend/api/core-client.mts",
        ],
    );
    assert!(file_paths(&extends)
        .iter()
        .any(|path| path.starts_with("packages/core/")));

    let base_url = run_json(
        &root,
        &[
            "dependencies",
            "--tsconfig",
            root_tsconfig.to_str().unwrap(),
            "--relationship",
            "import",
            "apps/backend/api/baseurl-client.mts",
        ],
    );
    assert!(file_paths(&base_url).contains(&"packages/core/src/internal.mts".to_string()));
}

#[test]
fn import_forms_report_expected_edge_kinds() {
    let root = fixture("import-forms");
    let cases = [
        ("static.mts", "import"),
        ("type-only.mts", "type-import"),
        ("inline-type.mts", "type-import"),
        ("import-type.mts", "type-import"),
        ("dynamic.mts", "dynamic-import"),
        ("require.js", "require"),
        ("reexport.mts", "import"),
    ];

    for (source, expected_kind) in cases {
        let value = run_json(&root, &["dependencies", "--relationship", "import", source]);
        assert_eq!(file_paths(&value), vec!["target.mts"]);
        assert_eq!(via_kinds(&value, "target.mts"), vec![expected_kind]);
    }

    let dependents = run_json(
        &root,
        &["dependents", "--relationship", "import", "target.mts"],
    );
    let mut paths = file_paths(&dependents);
    paths.sort();
    assert_eq!(
        paths,
        vec![
            "dynamic.mts",
            "import-type.mts",
            "inline-type.mts",
            "reexport.mts",
            "require.js",
            "static.mts",
            "type-only.mts",
        ]
    );
    assert_eq!(
        via_kinds(&dependents, "dynamic.mts"),
        vec!["dynamic-import"]
    );
    assert_eq!(via_kinds(&dependents, "require.js"), vec!["require"]);
    assert_eq!(
        via_kinds(&dependents, "inline-type.mts"),
        vec!["type-import"]
    );
}

#[test]
fn graph_edge_kind_acceptance() {
    let root = fixture("codebase-intel");

    let vitest = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "test",
            "--depth",
            "1",
            "packages/api/src/index.mts",
        ],
    );
    assert!(has_path_with_via(
        &vitest,
        "packages/api/src/index.test.mts",
        "test"
    ));

    let md = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "md",
            "--depth",
            "1",
            "README.md",
        ],
    );
    assert!(has_path_with_via(&md, "packages/api/src/index.mts", "md"));

    let process = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "process",
            "--depth",
            "1",
            "packages/api/src/spawn-runner.mts",
        ],
    );
    assert!(has_path_with_via(
        &process,
        "packages/api/src/spawn-target.mts",
        "process"
    ));

    let route = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "route",
            "--depth",
            "1",
            "packages/web/src/api-client.tsx",
        ],
    );
    assert!(has_path_with_via(
        &route,
        "packages/api/src/index.mts",
        "route"
    ));

    let route_reverse = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "route",
            "--depth",
            "1",
            "packages/api/src/index.mts",
        ],
    );
    assert!(has_path_with_via(
        &route_reverse,
        "packages/web/src/api-client.tsx",
        "route"
    ));

    let http = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "http",
            "--depth",
            "1",
            "packages/web/src/api-client.tsx",
        ],
    );
    assert!(has_path_with_via(
        &http,
        "packages/api/src/index.mts",
        "http"
    ));

    let http_reverse = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "http",
            "--depth",
            "1",
            "packages/api/src/index.mts",
        ],
    );
    assert!(has_path_with_via(
        &http_reverse,
        "packages/web/src/api-client.tsx",
        "http"
    ));

    let playwright = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "test",
            "--depth",
            "1",
            "tests/e2e/users.spec.ts",
        ],
    );
    assert!(has_path_with_via(
        &playwright,
        "packages/web/app/users/[id]/page.tsx",
        "route-test"
    ));

    let queue_enqueue = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "queue",
            "--depth",
            "1",
            "packages/api/src/send-email.mts",
        ],
    );
    assert!(has_queue_job_with_via(
        &queue_enqueue,
        "packages/api/src/emails.mts",
        "sendWelcomeEmail",
        "queue-enqueue"
    ));

    let queue_worker = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "queue",
            "--depth",
            "1",
            "packages/api/src/processors.mts",
        ],
    );
    assert!(has_queue_job_with_via(
        &queue_worker,
        "packages/api/src/emails.mts",
        "sendWelcomeEmail",
        "queue-worker"
    ));

    let ci = run_json(
        &root,
        &[
            "dependencies",
            "--relationship",
            "ci",
            "--depth",
            "1",
            ".github/workflows/ci.yml",
        ],
    );
    assert!(has_path_with_via(&ci, "src/bin/guardrails.rs", "ci"));
    assert!(has_path_with_via(&ci, "src/bin/pg_schema.rs", "ci"));

    let ci_reverse = run_json(
        &root,
        &[
            "dependents",
            "--relationship",
            "ci",
            "--depth",
            "1",
            "src/bin/guardrails.rs",
        ],
    );
    assert!(has_path_with_via(
        &ci_reverse,
        ".github/workflows/ci.yml",
        "ci"
    ));
}

#[test]
fn top_level_version_flag_prints_version() {
    let output = run(&["--version"]);
    assert_success(&output);
    let stdout = stdout(&output);
    let parts = stdout.split_whitespace().collect::<Vec<_>>();
    assert_eq!(parts.first(), Some(&"no-mistakes"));
    assert!(parts.get(1).is_some_and(|version| version.contains('.')));
}
