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
        sql_verdicts,
        include_sql_verdicts,
        suppressed,
        include_suppressed,
        timings,
        unproven_member_query_sql,
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
    if *include_sql_verdicts {
        value["sqlVerdicts"] = serde_json::to_value(sql_verdicts)
            .expect("SQL verdicts contain only serializable evidence");
    }
    if *include_suppressed {
        value["suppressed"] = serde_json::to_value(suppressed)
            .expect("suppression accounting serialization never fails");
    }
    if *unproven_member_query_sql > 0 {
        value["unprovenMemberQuerySql"] = serde_json::json!(unproven_member_query_sql);
    }
    // Dependency feature unification can switch serde_json maps from sorted
    // storage to insertion-ordered storage. Keep the public report stable in
    // either configuration, including keys in nested finding objects.
    value.sort_all_objects();
    value
}
