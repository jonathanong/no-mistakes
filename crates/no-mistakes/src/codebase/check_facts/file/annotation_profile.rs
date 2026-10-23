use super::CheckFactPlan;

pub(super) fn annotation_event(
    plan: &CheckFactPlan,
    options: &crate::codebase::postgres::EmbeddedSqlOptions,
    rel: &str,
) -> bool {
    let Some(scans) = plan.embedded_sql_rule_ids.get(options) else {
        return true;
    };
    let mut configured = false;
    let mut selected = false;
    for scan in scans {
        if scan.rule_id != "postgres-require-query-annotation" {
            continue;
        }
        configured = true;
        selected |= scan.paths.includes(rel);
    }
    // Profiles without this rule keep every collected file as a possible root.
    !configured || selected
}

pub(super) fn assign_unproven_rules(
    facts: &mut crate::codebase::postgres::EmbeddedSqlFileFacts,
    scans: Option<&[crate::codebase::postgres::EmbeddedSqlRuleScan]>,
    rel: &str,
    source: &str,
) {
    let Some(scans) = scans else {
        crate::codebase::postgres::retain_unproven_spans(facts, source);
        return;
    };
    let mut ids = Vec::new();
    for scan in scans {
        if scan.paths.includes(rel) && !ids.contains(&scan.rule_id) {
            ids.push(scan.rule_id.clone());
        }
    }
    if ids.is_empty() {
        facts.unproven_member_query_spans.clear();
        facts.record_unproven_spans = false;
        return;
    }
    facts.postgres_rule_ids = ids;
    crate::codebase::postgres::retain_unproven_spans(facts, source);
}
