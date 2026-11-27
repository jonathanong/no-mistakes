pub(super) fn merge_embedded_sql_rule_ids(
    existing: &mut crate::codebase::postgres_analysis::scans::EmbeddedSqlRuleScans,
    additional: crate::codebase::postgres_analysis::scans::EmbeddedSqlRuleScans,
) {
    for (options, scans) in additional {
        let slot = existing.entry(options).or_default();
        for scan in scans {
            if let Some(existing) = slot
                .iter_mut()
                .find(|existing| existing.same_application(&scan))
            {
                existing.suppression_audit |= scan.suppression_audit;
            } else {
                slot.push(scan);
            }
        }
    }
}
