use std::sync::Arc;
use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Default)]
pub struct CheckFactPlan {
    pub imports: bool,
    /// Explicit production-rule sources, including metadata and unknown extensions.
    /// Their import projection preserves the historical TypeScript/TSX parse mode.
    #[doc(hidden)]
    pub production_import_paths: Arc<std::collections::BTreeSet<std::path::PathBuf>>,
    #[doc(hidden)]
    pub companion_import_plans: Arc<
        Vec<Arc<crate::codebase::rules::required_companion_imports::PreparedCompanionImportPlan>>,
    >,
    pub symbols: bool,
    #[doc(hidden)]
    pub legacy_symbol_paths: HashSet<PathBuf>,
    pub react: bool,
    pub react_usages: bool,
    pub queue: bool,
    pub queue_factory_names: Vec<String>,
    pub queue_factory_roles: crate::config::v2::schema::QueueFactoryRoles,
    pub integration: bool,
    pub integration_runner_configs:
        Option<Arc<crate::integration_tests::PreparedIntegrationRunnerConfigs>>,
    pub dynamic_imports: bool,
    pub nextjs_caching: bool,
    pub storybook: bool,
    pub server_route_client_boundary: bool,
    pub source: bool,
    pub raw_source: bool,
    pub postgres_schema: bool,
    pub embedded_sql: bool,
    /// Owned helper/callback summaries for the annotation-only projection.
    pub query_annotation: bool,
    /// Explicit annotation-only profiles. `None` preserves the direct fact
    /// API's request for annotation on every embedded SQL profile.
    pub query_annotation_options: Option<Vec<crate::codebase::postgres::EmbeddedSqlOptions>>,
    /// The owning request's ordinary importer catalog, including explicit overrides.
    #[doc(hidden)]
    pub query_annotation_catalog: Option<Arc<crate::codebase::ts_resolver::TsConfigCatalog>>,
    /// Distinct configured embedded-SQL projections collected from each
    /// already-parsed TS/JS program during the request's shared fact pass.
    pub embedded_sql_options: Vec<crate::codebase::postgres::EmbeddedSqlOptions>,
    /// PostgreSQL rule applications that selected each embedded-SQL profile.
    ///
    /// Empty for a direct extract. Each entry carries that application's
    /// include and exclude. A site is omitted when no application scans the
    /// file, or when every scanning id suppresses that line.
    pub embedded_sql_rule_ids: crate::codebase::postgres::EmbeddedSqlRuleScans,
    /// Repository-relative schema snapshots loaded once at the request boundary.
    pub postgres_schema_catalog_paths: Vec<String>,
    pub postgres_dml: bool,
    /// Demand concrete alternatives for rules that retain legacy direct SQL handling.
    pub postgres_variants: bool,
    /// Opted-in row-bound projection within the shared SQL statement fact pass.
    pub postgres_bounds: bool,
    /// Opted-in builder-fragment statement facts for shape-policy consumers.
    pub postgres_fragments: bool,
    /// Query-file globs for write facts, independent of migration schema scope.
    pub postgres_write_sql_include: Vec<String>,
    /// Union of SQL globs resolved from configured rule defaults. Empty selects no SQL files.
    pub postgres_sql_include: Vec<String>,
    pub graph: crate::codebase::ts_source::facts::TsFactPlan,
    pub graph_context: crate::codebase::ts_source::facts::TsFactContext,
}

fn merge_embedded_sql_rule_ids(
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

impl CheckFactPlan {
    pub(super) fn with_production_files(
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

    pub(crate) fn include(&mut self, other: Self) {
        self.imports |= other.imports;
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
        self.symbols |= other.symbols;
        self.legacy_symbol_paths.extend(other.legacy_symbol_paths);
        self.react |= other.react;
        self.react_usages |= other.react_usages;
        self.queue |= other.queue;
        self.queue_factory_names.extend(other.queue_factory_names);
        self.queue_factory_names.sort();
        self.queue_factory_names.dedup();
        self.queue_factory_roles.merge(&other.queue_factory_roles);
        self.integration |= other.integration;
        if self.integration_runner_configs.is_none() {
            self.integration_runner_configs = other.integration_runner_configs;
        }
        self.dynamic_imports |= other.dynamic_imports;
        self.nextjs_caching |= other.nextjs_caching;
        self.storybook |= other.storybook;
        self.server_route_client_boundary |= other.server_route_client_boundary;
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
        self.postgres_bounds |= other.postgres_bounds;
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

    /// TS graph fact shapes guaranteed for every file collected with this
    /// plan. Several check-only flags populate the canonical `TsFileFacts`
    /// fields too; recording only `graph` would make a strict prepared graph
    /// reject facts that were already collected in the same parse pass.
    pub(super) fn collected_ts_plan(&self) -> crate::codebase::ts_source::facts::TsFactPlan {
        let mut plan = self.graph;
        plan.imports |=
            self.query_annotation || self.embedded_sql || !self.embedded_sql_options.is_empty();
        if self.imports {
            plan.include(crate::codebase::ts_source::facts::TsFactPlan::imports());
        }
        plan.symbols |= self.symbols;
        plan.source |= self.source || self.raw_source;
        plan.react |= self.react;
        plan.queue_project |= self.queue;
        plan
    }
}

#[cfg(test)]
mod tests;
