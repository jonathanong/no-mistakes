use super::CheckResults;

pub(crate) fn json_value(results: &CheckResults) -> serde_json::Value {
    let CheckResults {
        react,
        queues,
        rules,
        integration,
        runner_config_deadlines,
        codebase,
        warnings,
        advisories,
        suppressed,
        include_suppressed,
        timings,
    } = results;
    let _ = timings;
    let mut value = serde_json::json!({
        "react": react,
        "queues": queues,
        "rules": rules,
        "integration": integration,
        "codebase": codebase,
        "warnings": warnings,
        "advisories": advisories,
    });
    if let Some(evidence) = runner_config_deadlines {
        value["runnerConfigDeadlines"] = serde_json::to_value(evidence)
            .expect("finite declared deadline evidence serialization never fails");
    }
    if *include_suppressed {
        value["suppressed"] = serde_json::to_value(suppressed)
            .expect("suppression accounting serialization never fails");
    }
    // Dependency feature unification can switch serde_json maps from sorted
    // storage to insertion-ordered storage. Keep the public report stable in
    // either configuration, including keys in nested finding objects.
    value.sort_all_objects();
    value
}
