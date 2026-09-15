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
