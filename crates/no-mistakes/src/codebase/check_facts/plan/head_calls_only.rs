use super::CheckFactPlan;
use crate::codebase::postgres::EmbeddedSqlOptions;

impl CheckFactPlan {
    pub(in crate::codebase::check_facts) fn with_production_files(
        &self,
        mut files: Vec<std::path::PathBuf>,
    ) -> Vec<std::path::PathBuf> {
        if !self.production_import_paths.is_empty() {
            files.extend(self.production_import_paths.iter().cloned());
            files.sort();
            files.dedup();
        }
        files
    }

    pub(super) fn merge_head_calls_only(&mut self, other: &Self) {
        // Full demand wins, including a direct library request without rule scans.
        let full = |plan: &Self, profile: &EmbeddedSqlOptions| {
            let direct_full = plan.embedded_sql_rule_ids.is_empty()
                && (plan.postgres_dml || plan.postgres_fragments || plan.postgres_variants);
            let evaluated = plan.query_annotation
                && plan
                    .query_annotation_options
                    .as_ref()
                    .is_none_or(|profiles| profiles.contains(profile));
            direct_full
                || evaluated
                || ((plan.embedded_sql_options.contains(profile)
                    || (plan.embedded_sql
                        && plan.embedded_sql_options.is_empty()
                        && profile == &EmbeddedSqlOptions::default()))
                    && !plan.embedded_sql_head_calls_only.contains(profile))
        };
        let mut candidates = self.embedded_sql_head_calls_only.clone();
        candidates.extend(other.embedded_sql_head_calls_only.iter().cloned());
        candidates.retain(|profile| !full(self, profile) && !full(other, profile));
        candidates.sort();
        candidates.dedup();
        self.embedded_sql_head_calls_only = candidates;
    }
}

#[cfg(test)]
mod tests;
