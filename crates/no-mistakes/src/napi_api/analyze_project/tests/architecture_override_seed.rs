use super::*;
use serde_json::json;
use std::path::PathBuf;

#[test]
fn mixed_all_and_dependents_reports_build_one_canonical_graph() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/performance/core-analysis");
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let output = {
        let _guard = crate::diagnostics::InvocationGuard::install(observer.clone());
        analyze_project_json_impl(crate::napi_api::options::test_json_arg(
            json!({
                "root": root,
                "tsconfig": root.join("tsconfig.json"),
                "reports": [
                    {
                        "id": "dependencies",
                        "type": "dependencies",
                        "files": ["src/app.tsx"],
                        "relationships": ["all"]
                    },
                    {
                        "id": "dependents",
                        "type": "dependents",
                        "files": ["packages/core/src/index.ts"]
                    },
                    {
                        "id": "symbol-dependents",
                        "type": "dependents",
                        "files": [{
                            "file": "packages/core/src/index.ts",
                            "symbol": "CoreValue"
                        }],
                        "relationships": ["all"]
                    },
                    {
                        "id": "symbols",
                        "type": "symbols",
                        "files": ["src/app.tsx"],
                        "include": "both"
                    }
                ]
            })
            .to_string(),
        ))
        .unwrap()
    };
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value["reports"].as_array().map(Vec::len), Some(4));
    let work = observer.snapshot().work;
    assert_eq!(work["graph.builds"], 1, "{work:#?}");
    assert!(work["graph.reuses"] >= 1, "{work:#?}");
}

#[test]
fn mixed_graph_reports_seed_canonical_graph_before_parallel_projection() {
    let seed = include_str!("../context/scope_seed_import.rs");
    assert!(
        seed.contains("seed_canonical_graph_if_needed") && seed.contains("graph_shared"),
        "non-import-only analyzeProject graph reports must seed the canonical graph"
    );
    let prepare = include_str!("../context/api.rs");
    assert!(
        prepare.contains("seed_canonical_graph_if_needed"),
        "prepare must seed the canonical graph before report execution"
    );
    let dispatch = include_str!("../../analyze_project.rs");
    let body = dispatch
        .split("fn analyze_project(")
        .nth(1)
        .and_then(|source| source.split("fn run_report(").next())
        .expect("analyze_project is defined");
    assert!(
        body.contains("AnalyzeProjectContext::prepare") && body.contains("par_iter"),
        "analyze_project must prepare the shared context before parallel projection"
    );
    let prepare_at = body
        .find("AnalyzeProjectContext::prepare")
        .expect("analyze_project prepares a shared context");
    let par_iter_at = body
        .find("par_iter")
        .expect("analyze_project runs reports in parallel");
    assert!(
        prepare_at < par_iter_at,
        "canonical graph seed runs during prepare, before report par_iter"
    );
}
