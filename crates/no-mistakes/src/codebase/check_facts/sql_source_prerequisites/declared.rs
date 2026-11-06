use super::{SqlProjection, SqlSourceKey, SqlSourcePrerequisites};
use crate::codebase::{check_facts::CheckFactPlan, postgres::EmbeddedSqlOptions};
use std::path::Path;

pub(crate) fn planned_low_applications(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
) -> Vec<(EmbeddedSqlOptions, String, usize)> {
    if !crate::codebase::dependencies::extract::is_indexable(path) {
        return Vec::new();
    }
    plan.sql_identifier_profiles
        .iter()
        .flat_map(|(profile, scans)| {
            scans
                .iter()
                .filter(|scan| scan.includes(root, path))
                .filter_map(|scan| {
                    scan.application_index
                        .map(|index| (profile.clone(), scan.rule.clone(), index))
                })
        })
        .collect()
}

impl SqlSourcePrerequisites {
    pub(crate) fn failed_source(root: &Path, path: &Path, plan: &CheckFactPlan) -> Self {
        let mut declared = std::collections::BTreeSet::new();
        for (profile, rule, index) in planned_low_applications(root, path, plan) {
            declared.insert(SqlSourceKey::application(
                SqlProjection::LowSyntax,
                &profile,
                &rule,
                index,
            ));
        }
        if plan.query_annotation && crate::codebase::dependencies::extract::is_indexable(path) {
            let defaults;
            let profiles = match &plan.query_annotation_options {
                Some(profiles) => profiles,
                None => {
                    defaults = if plan.embedded_sql && plan.embedded_sql_options.is_empty() {
                        vec![EmbeddedSqlOptions::default()]
                    } else {
                        plan.embedded_sql_options.clone()
                    };
                    &defaults
                }
            };
            declared.extend(
                profiles
                    .iter()
                    .map(|profile| SqlSourceKey::profile(SqlProjection::AnnotationMax, profile)),
            );
        }
        Self {
            declared: Some(declared),
            ..Self::default()
        }
    }
}

pub(crate) fn with_failed_source(
    root: &Path,
    path: &Path,
    plan: &CheckFactPlan,
    mut facts: super::CheckFileFacts,
) -> super::CheckFileFacts {
    facts.sql_source_prerequisites = SqlSourcePrerequisites::failed_source(root, path, plan);
    facts.sql_identifier_syntax.low_applications = planned_low_applications(root, path, plan);
    facts
}
