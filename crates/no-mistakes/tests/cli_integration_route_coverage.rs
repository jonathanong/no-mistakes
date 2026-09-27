use std::path::PathBuf;
use std::process::Command;

fn root() -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/integration-route-coverage"),
    )
}

#[test]
fn route_changes_select_the_real_integration_entry_in_generic_and_vitest_plans() {
    let root = root();
    for framework in [None, Some("vitest")] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_no-mistakes"));
        command.args(["tests", "plan"]);
        if let Some(framework) = framework {
            command.arg(framework);
        }
        let output = command
            .args([
                "--root",
                root.to_str().unwrap(),
                "--changed-file",
                "web/app/healthz/page.tsx",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["fallback_triggered"], false, "{report}");
        let selected = report["selected_tests"].as_array().unwrap();
        assert_eq!(selected.len(), 1, "{report}");
        assert_eq!(
            selected[0]["test_file"], "integration/web.test.ts",
            "{report}"
        );
        assert!(
            selected[0]["reasons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reason| reason["via"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|kind| kind == "route")),
            "{report}"
        );
    }
}
