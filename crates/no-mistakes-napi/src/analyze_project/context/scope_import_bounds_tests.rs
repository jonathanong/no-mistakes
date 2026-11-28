use super::*;

#[test]
fn equivalent_bounds_union_roots_and_keep_the_widest_requested_depth() {
    let prepared = options(json!([
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"],
            "candidateInclude":["*.mts", "a.mts"], "depth":1},
        {"type":"dependencies", "files":["b.mts"], "relationships":["import"],
            "candidateInclude":["a.mts", "*.mts", "a.mts"], "depth":3}
    ]));
    let plan = classify_import_seed_requests(&prepared).unwrap();
    assert!(plan.unbounded.is_none());
    assert!(!plan.other_graph_consumer);
    assert_eq!(plan.bounded.len(), 1);
    let bounds = plan.exclusive_bounds().unwrap();
    assert_eq!(
        bounds.files,
        [PathBuf::from("a.mts"), PathBuf::from("b.mts")]
    );
    assert_eq!(bounds.depth, Some(3));

    let mut unlimited = prepared.clone();
    unlimited.reports[1].options.remove("depth");
    let plan = classify_import_seed_requests(&unlimited).unwrap();
    assert_eq!(plan.exclusive_bounds().unwrap().depth, None);
}

fn report_files(
    context: &AnalyzeProjectContext,
    prepared: &AnalyzeProjectOptions,
    index: usize,
) -> Vec<String> {
    let report = context
        .graph_report(&prepared.reports[index], prepared, Direction::Deps)
        .unwrap();
    let report: Value = serde_json::from_str(report.get()).unwrap();
    report["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn bounded_and_unbounded_siblings_keep_reachable_imports_outside_the_candidates() {
    let prepared = options(json!([
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"],
            "candidateInclude":["a.mts"], "candidateExclude":["c.mts"], "projection":"paths"},
        {"type":"dependencies", "files":["b.mts"], "relationships":["import"],
            "candidateInclude":["b.mts"], "projection":"paths"},
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"], "projection":"paths"}
    ]));
    let plan = classify_import_seed_requests(&prepared).unwrap();
    assert_eq!(plan.bounded.len(), 2);
    assert!(plan.unbounded.is_some());
    assert!(plan.exclusive_bounds().is_none());
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    let bounded = report_files(&context, &prepared, 0);
    assert!(bounded.iter().any(|file| file == "c.mts"), "{bounded:?}");
    assert_eq!(bounded, report_files(&context, &prepared, 2));
    let sibling = report_files(&context, &prepared, 1);
    assert!(sibling.iter().any(|file| file == "c.mts"), "{sibling:?}");
    assert!(!sibling.iter().any(|file| file == "a.mts"), "{sibling:?}");
}

#[test]
fn exclusive_candidates_preserve_import_escape_and_per_report_depth() {
    let prepared = options(json!([
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"],
            "candidateInclude":["a.mts"], "candidateExclude":["c.mts"], "projection":"paths", "depth":1},
        {"type":"dependencies", "files":["a.mts"], "relationships":["import"],
            "candidateInclude":["a.mts"], "candidateExclude":["c.mts"], "projection":"paths"}
    ]));
    let context = AnalyzeProjectContext::prepare(&prepared).unwrap();
    assert_eq!(context.scopes.len(), 1);
    let scope = context.scopes.values().next().unwrap();
    assert!(scope.traversal.candidate_inventory_applied());
    let shallow = report_files(&context, &prepared, 0);
    let complete = report_files(&context, &prepared, 1);
    assert!(shallow.iter().any(|file| file == "b.mts"), "{shallow:?}");
    assert!(!shallow.iter().any(|file| file == "c.mts"), "{shallow:?}");
    assert!(complete.iter().any(|file| file == "c.mts"), "{complete:?}");
    assert_eq!(complete, report_files(&context, &prepared, 1));
}

#[test]
fn prepared_reports_reject_bounds_that_change_relationship_or_direction_semantics() {
    for (report, message) in [
        (
            json!({"type":"dependents", "files":["a.mts"], "candidateInclude":["*.mts"]}),
            "only valid for dependencies",
        ),
        (
            json!({"type":"dependencies", "files":["a.mts"], "candidateInclude":["*.mts"], "includeSymbols":true}),
            "cannot be combined with includeSymbols",
        ),
        (
            json!({"type":"dependencies", "files":["a.mts"], "candidateExclude":["b.mts"], "relationships":["call"]}),
            "require import-only relationships",
        ),
        (
            json!({"type":"dependencies", "files":["a.mts"], "candidateInclude":["["], "relationships":["import"]}),
            "candidateInclude",
        ),
    ] {
        let prepared = options(json!([report]));
        let error = AnalyzeProjectContext::prepare(&prepared)
            .err()
            .expect("invalid bounds rejected");
        assert!(format!("{error:#}").contains(message), "{error:#}");
    }
}
