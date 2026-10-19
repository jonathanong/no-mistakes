pub(super) fn merge_embedded_sql_rule_ids(
    existing: &mut crate::codebase::postgres::EmbeddedSqlRuleScans,
    additional: crate::codebase::postgres::EmbeddedSqlRuleScans,
) {
    for (options, scans) in additional {
        let slot = existing.entry(options).or_default();
        for scan in scans {
            if !slot.iter().any(|existing| existing.same_application(&scan)) {
                slot.push(scan);
            }
        }
    }
}
