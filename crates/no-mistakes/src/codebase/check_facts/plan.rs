mod call_sites;
mod collected;
mod location_demand;
mod rule_scans;
use std::sync::Arc;
use std::{collections::HashSet, path::PathBuf};

#[derive(Clone, Default)]
pub struct CheckFactPlan {
    pub imports: bool,
    #[doc(hidden)]
    pub postgres_executor_warning_scans:
        Arc<Vec<crate::codebase::postgres_analysis::executor_warning_scan::ExecutorWarningScan>>,
    #[doc(hidden)]
    pub nested_control_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    /// Exact configured files whose AST comments are requested.
    pub lint_suppression_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    #[doc(hidden)]
    pub vitest_default_exclude_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    #[doc(hidden)]
    pub csp_allowed_origins_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    #[doc(hidden)]
    pub test_reliability_paths: Arc<
        std::collections::BTreeMap<
            PathBuf,
            crate::codebase::testing_analysis::reliability::SourceDemand,
        >,
    >,
    #[doc(hidden)]
    pub playwright_fixture_support: bool,
    /// Exact call-source demand; broad `graph.call_sites` remains independent.
    #[doc(hidden)]
    pub call_site_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    #[doc(hidden)]
    pub forbidden_call_arguments:
        Option<Arc<crate::codebase::rules::call_policy::ArgumentFactScope>>,
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
    pub symbols_paths: super::FamilyPathScope,
    #[doc(hidden)]
    pub legacy_symbol_paths: HashSet<PathBuf>,
    pub react: bool,
    #[doc(hidden)]
    pub react_paths: super::FamilyPathScope,
    pub react_usages: bool,
    pub queue: bool,
    #[doc(hidden)]
    pub queue_paths: super::FamilyPathScope,
    pub queue_factory_names: Vec<String>,
    pub queue_factory_roles: crate::config::v2::schema::QueueFactoryRoles,
    pub integration: bool,
    #[doc(hidden)]
    pub integration_paths: super::FamilyPathScope,
    pub integration_runner_configs:
        Option<Arc<crate::integration_tests::PreparedIntegrationRunnerConfigs>>,
    pub dynamic_imports: bool,
    pub nextjs_caching: bool,
    pub nextjs_caching_low: bool,
    #[doc(hidden)]
    pub nextjs_caching_paths: super::FamilyPathScope,
    pub storybook: bool,
    #[doc(hidden)]
    pub storybook_paths: super::FamilyPathScope,
    pub server_route_client_boundary: bool,
    #[doc(hidden)]
    pub boundary_paths: super::FamilyPathScope,
    pub source: bool,
    pub raw_source: bool,
    /// Only route candidates require raw source for configured checks. Direct
    /// fact requests keep the historical whole-file default.
    #[doc(hidden)]
    pub raw_source_paths: super::FamilyPathScope,
    pub postgres_schema: bool,
    #[doc(hidden)]
    pub postgres_explain_write_rollback_paths: Arc<std::collections::BTreeSet<PathBuf>>,
    pub embedded_sql: bool,
    /// Owned helper/callback summaries for the annotation-only projection.
    pub query_annotation: bool,
    /// Syntactic leading-name proofs for low-effort annotation, without helper evaluation.
    pub query_annotation_heads: bool,
    /// Low-effort SQL identifier projections; never invokes JavaScript evaluation.
    pub sql_identifier_profiles:
        crate::codebase::postgres_analysis::identifier_plan::SqlIdentifierScans,
    /// Explicit annotation-only profiles. `None` preserves the direct fact
    /// API's request for annotation on every embedded SQL profile.
    pub query_annotation_options:
        Option<Vec<crate::codebase::postgres_analysis::options::EmbeddedSqlOptions>>,
    /// The owning request's ordinary importer catalog, including explicit overrides.
    #[doc(hidden)]
    pub query_annotation_catalog: Option<Arc<crate::codebase::ts_resolver::TsConfigCatalog>>,
    /// Distinct configured embedded-SQL projections collected from each
    /// already-parsed TS/JS program during the request's shared fact pass.
    pub embedded_sql_options: Vec<crate::codebase::postgres_analysis::options::EmbeddedSqlOptions>,
    /// Profiles consumed exclusively by syntactic, low-effort annotation checks.
    #[doc(hidden)]
    pub embedded_sql_head_calls_only:
        Vec<crate::codebase::postgres_analysis::options::EmbeddedSqlOptions>,
    /// PostgreSQL rule applications that selected each embedded-SQL profile.
    ///
    /// Empty for a direct extract. Each entry carries that application's
    /// include and exclude. A site is omitted when no application scans the
    /// file, or when every scanning id suppresses that line.
    pub embedded_sql_rule_ids: crate::codebase::postgres_analysis::scans::EmbeddedSqlRuleScans,
    /// Repository-relative schema snapshots loaded once at the request boundary.
    pub postgres_schema_catalog_paths: Vec<String>,
    pub postgres_dml: bool,
    /// Demand concrete alternatives for rules that retain legacy direct SQL handling.
    pub postgres_variants: bool,
    /// Exact executor SQL needs source-aware lock/conflict metadata in the shared parse.
    pub postgres_exact_locations: bool,
    /// Conflict-only exact SQL location projection, unless a broader consumer joins.
    pub postgres_conflict_locations: bool,
    /// General SQL consumers require Full when sharing conflict-location demand.
    pub postgres_general_locations: bool,
    /// Opted-in row-bound projection within the shared SQL statement fact pass.
    pub postgres_bounds: bool,
    pub postgres_lock_inventory: bool,
    /// Literal JSON relation keys, collected only for their configured consumer.
    pub postgres_json_writes: bool,
    pub postgres_duplicate_keyed_lookups: bool,
    pub postgres_grouping: bool,
    pub postgres_index_probes: bool,
    pub postgres_body_bounds: bool,
    /// Opted-in builder-fragment statement facts for shape-policy consumers.
    pub postgres_fragments: bool,
    /// Query-file globs for write facts, independent of migration schema scope.
    pub postgres_write_sql_include: Vec<String>,
    /// Union of SQL globs resolved from configured rule defaults. Empty selects no SQL files.
    pub postgres_sql_include: Vec<String>,
    pub graph: crate::codebase::ts_source::facts::TsFactPlan,
    pub graph_context: crate::codebase::ts_source::facts::TsFactContext,
}
mod merge;

mod head_calls_only;
#[cfg(all(test, any(nm_shard_all, nm_shard_1)))]
mod tests;

mod lint_suppressions;
