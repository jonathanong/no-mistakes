pub(super) fn head_calls_only_profiles(
    options: &[EmbeddedSqlOptions],
    annotation: &[EmbeddedSqlOptions],
    scans: &super::scans::EmbeddedSqlRuleScans,
) -> Vec<EmbeddedSqlOptions> {
    options
        .iter()
        .filter(|profile| {
            !annotation.contains(profile)
                && scans.get(*profile).is_some_and(|scans| {
                    !scans.is_empty()
                        && scans
                            .iter()
                            .all(|scan| scan.rule_id == "postgres-require-query-annotation")
                })
        })
        .cloned()
        .collect()
}
