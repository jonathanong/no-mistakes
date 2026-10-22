pub(super) fn merge_embedded_sql_rule_ids(
    existing: &mut super::CheckFactPlan,
    additional: &mut super::CheckFactPlan,
) {
    if existing.embedded_sql_rule_ids.is_empty() && additional.embedded_sql_rule_ids.is_empty() {
        return;
    }
    // A direct plan's unspecified scope dominates any configured application
    // selecting the same profile, regardless of merge order.
    let unrestricted: Vec<_> = [existing as &super::CheckFactPlan, additional]
        .into_iter()
        .flat_map(|plan| {
            let mut profiles = plan.embedded_sql_options.clone();
            if plan.embedded_sql && profiles.is_empty() {
                profiles.push(Default::default());
            }
            profiles
                .into_iter()
                .filter(move |profile| !plan.embedded_sql_rule_ids.contains_key(profile))
        })
        .collect();
    for (options, scans) in std::mem::take(&mut additional.embedded_sql_rule_ids) {
        let slot = existing.embedded_sql_rule_ids.entry(options).or_default();
        for scan in scans {
            if !slot.iter().any(|existing| existing.same_application(&scan)) {
                slot.push(scan);
            }
        }
    }
    for profile in unrestricted {
        existing.embedded_sql_rule_ids.remove(&profile);
    }
}

impl super::CheckFactPlan {
    pub(crate) fn scans_embedded_sql(
        &self,
        profile: &crate::codebase::postgres::EmbeddedSqlOptions,
        relative_path: &str,
    ) -> bool {
        self.embedded_sql_rule_ids
            .get(profile)
            .is_none_or(|scans| scans.iter().any(|scan| scan.paths.includes(relative_path)))
    }
}
