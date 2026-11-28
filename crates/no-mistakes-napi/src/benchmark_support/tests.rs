use super::*;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn analyze_project_benchmark_adapters_preserve_output_with_and_without_observers() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/core-analysis")
        .canonicalize()
        .expect("performance fixture should exist");
    let options = json!({
        "root": root,
        "tsconfig": root.join("tsconfig.json"),
        "reports": [
            {
                "id": "dependencies",
                "type": "dependencies",
                "files": ["src/app.tsx"],
                "relationships": ["all"]
            }
        ]
    })
    .to_string();
    let plain_project =
        analyze_project_json(options.clone()).expect("plain project analysis should succeed");
    let (observed_project, project_diagnostics) =
        analyze_project_json_observed(options).expect("observed project analysis should succeed");
    assert_eq!(observed_project, plain_project);
    assert!(!project_diagnostics.work.is_empty());
}
