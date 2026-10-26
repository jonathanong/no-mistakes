use super::{rule_scans::merge_embedded_sql_rule_ids, CheckFactPlan};
use std::sync::Arc;

impl CheckFactPlan {
    pub(crate) fn include(&mut self, other: Self) {
        self.forbidden_call_arguments =
            crate::codebase::rules::forbidden_calls::ArgumentFactScope::merge(
                self.forbidden_call_arguments.as_ref(),
                other.forbidden_call_arguments.as_ref(),
            );
        self.merge_head_calls_only(&other);
        self.merge_location_demand(&other);
        super::super::family_scope::include_scopes(self, &other);
        self.merge_call_site_paths(&other);
        if !other.lint_suppression_paths.is_empty() {
            Arc::make_mut(&mut self.lint_suppression_paths)
                .extend(other.lint_suppression_paths.iter().cloned());
        }
        self.imports |= other.imports;
        self.query_annotation_heads |= other.query_annotation_heads;
        crate::codebase::postgres::sql_identifier_syntax::plan::merge(
            &mut self.sql_identifier_profiles,
            other.sql_identifier_profiles,
        );
        self.companion_import_plans =
            crate::codebase::rules::required_companion_imports::merge_plans(
                &self.companion_import_plans,
                &other.companion_import_plans,
            );
        if !other.production_import_paths.is_empty() {
            if self.production_import_paths.is_empty() {
                self.production_import_paths = other.production_import_paths;
            } else if !Arc::ptr_eq(
                &self.production_import_paths,
                &other.production_import_paths,
            ) {
                Arc::make_mut(&mut self.production_import_paths)
                    .extend(other.production_import_paths.iter().cloned());
            }
        }
        if other.query_annotation {
            if self.query_annotation {
                self.query_annotation_options = match (
                    self.query_annotation_options.take(),
                    other.query_annotation_options,
                ) {
                    (Some(mut existing), Some(additional)) => {
                        existing.extend(additional);
                        existing.sort();
                        existing.dedup();
                        Some(existing)
                    }
                    _ => None,
                };
            } else {
                self.query_annotation_options = other.query_annotation_options;
            }
            self.query_annotation = true;
        }
        if self.query_annotation_catalog.is_none() {
            self.query_annotation_catalog = other.query_annotation_catalog;
        }
        self.legacy_symbol_paths.extend(other.legacy_symbol_paths);
        self.react_usages |= other.react_usages;
        self.queue_factory_names.extend(other.queue_factory_names);
        self.queue_factory_names.sort();
        self.queue_factory_names.dedup();
        // Config load already rejects conflicting roles, so a merge error keeps the first plan.
        let merged = self.queue_factory_roles.merge(&other.queue_factory_roles);
        debug_assert!(
            merged.is_ok(),
            "conflicting queue factory roles: {merged:?}"
        );
        if self.integration_runner_configs.is_none() {
            self.integration_runner_configs = other.integration_runner_configs;
        }
        self.dynamic_imports |= other.dynamic_imports;
        self.nextjs_caching |= other.nextjs_caching;
        self.nextjs_caching_low |= other.nextjs_caching_low;
        self.source |= other.source;
        self.raw_source |= other.raw_source;
        self.postgres_schema |= other.postgres_schema;
        self.embedded_sql |= other.embedded_sql;
        self.embedded_sql_options.extend(other.embedded_sql_options);
        self.embedded_sql_options.sort();
        self.embedded_sql_options.dedup();
        merge_embedded_sql_rule_ids(&mut self.embedded_sql_rule_ids, other.embedded_sql_rule_ids);
        self.postgres_schema_catalog_paths
            .extend(other.postgres_schema_catalog_paths);
        self.postgres_schema_catalog_paths.sort();
        self.postgres_schema_catalog_paths.dedup();
        self.postgres_dml |= other.postgres_dml;
        self.postgres_variants |= other.postgres_variants;
        self.postgres_exact_locations |= other.postgres_exact_locations;
        self.postgres_conflict_locations |= other.postgres_conflict_locations;
        self.postgres_bounds |= other.postgres_bounds;
        self.postgres_json_writes |= other.postgres_json_writes;
        self.postgres_fragments |= other.postgres_fragments;
        self.postgres_write_sql_include
            .extend(other.postgres_write_sql_include);
        self.postgres_write_sql_include.sort();
        self.postgres_write_sql_include.dedup();
        self.postgres_sql_include.extend(other.postgres_sql_include);
        self.postgres_sql_include.sort();
        self.postgres_sql_include.dedup();
        self.graph.include(other.graph);
        self.graph_context.include(other.graph_context);
    }
}
