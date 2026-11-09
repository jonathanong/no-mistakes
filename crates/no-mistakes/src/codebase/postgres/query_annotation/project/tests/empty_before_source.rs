use super::demand_followups::{fixture, observed_check};

#[test]
fn source_join_skips_only_the_after_walk_when_the_before_slots_are_unannotated() {
    let (report, diagnostics) = observed_check(&fixture("empty-before-prefix-source"));
    assert_eq!(report["warnings"], serde_json::json!([]), "{report}");
    let findings = report["rules"].as_array().expect("rule findings");
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding["line"].as_u64().expect("finding line"))
            .collect::<Vec<_>>(),
        [12, 13, 17],
        "dense and sparse callback findings plus the explicit unsafe query: {report}"
    );
    assert!(
        findings.iter().all(|finding| {
            finding["file"] == "src/query.mts"
                && finding["message"].as_str().is_some_and(|message| {
                    message.ends_with("query execution must start with a /* name */ annotation")
                })
        }),
        "all expected calls retain the annotation violation: {report}"
    );

    let work = diagnostics.work;
    assert!(
        work["postgres.annotation.prefix_after_empty_skips"] > 0,
        "{work:?}"
    );
    assert!(work["postgres.annotation.prefix_walks"] > 0, "{work:?}");
    assert_eq!(
        work["postgres.annotation.prefix_walks_completed"],
        work["postgres.annotation.prefix_walks"],
        "skipped after walks still count as completed comparisons: {work:?}"
    );
    assert!(
        work["postgres.annotation.prefix_after_empty_skips"]
            <= work["postgres.annotation.prefix_walks"],
        "at most one after walk can be skipped per comparison: {work:?}"
    );
}
