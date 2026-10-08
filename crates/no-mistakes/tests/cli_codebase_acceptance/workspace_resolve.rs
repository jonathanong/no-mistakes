use super::common::{assert_success, fixture, run_in, stdout};
use serde_json::Value;

#[test]
fn workspace_resolve_check_cli_fails_closed_and_matches_workspace_graph() {
    let root = fixture("workspace-resolve-check");
    let root_arg = root.to_string_lossy();
    for force_tsconfig in [false, true] {
        let mut args = vec![
            "resolve-check",
            "packages/app/entry.mts",
            "--root",
            &root_arg,
            "--format",
            "json",
        ];
        if force_tsconfig {
            args.extend(["--tsconfig", "tsconfig.json"]);
        }
        let output = run_in(&root, &args);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_str(&stdout(&output)).unwrap();
        assert_eq!(report["allResolve"], false);
        let rows = report["imports"].as_array().unwrap();
        let row = |specifier: &str| {
            rows.iter()
                .find(|row| row["specifier"] == specifier)
                .unwrap()
        };
        assert_eq!(row("@fx/lib/target")["resolved"], "packages/lib/target.mts");
        assert_eq!(row("@fx/lib/missing")["status"], "unresolved");
        assert_eq!(row("@fx/lib/missing-dynamic")["status"], "unresolved");
        assert_eq!(row("@fx/not-a-package")["status"], "external");
        assert_eq!(row("third-party")["status"], "external");
        args[1] = "packages/app/valid.mts";
        let output = run_in(&root, &args);
        assert_success(&output);
        let report: Value = serde_json::from_str(&stdout(&output)).unwrap();
        let expected = report["imports"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["resolved"].as_str().unwrap().to_string())
            .collect::<std::collections::BTreeSet<_>>();
        let mut args = vec![
            "dependencies",
            "packages/app/valid.mts",
            "--root",
            &root_arg,
            "--relationship",
            "workspace",
            "--depth",
            "10",
            "--projection",
            "paths",
            "--format",
            "json",
        ];
        if force_tsconfig {
            args.extend(["--tsconfig", "tsconfig.json"]);
        }
        let output = run_in(&root, &args);
        assert_success(&output);
        let graph: Value = serde_json::from_str(&stdout(&output)).unwrap();
        let actual = graph["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|path| path.as_str().unwrap().replace('\\', "/"))
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(actual, expected);
    }
}
