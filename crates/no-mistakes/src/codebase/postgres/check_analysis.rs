pub(crate) mod file;
pub(crate) mod map;
pub(crate) use map::prepare;
pub(crate) mod relative_scope;
pub(crate) mod source_demand;

pub use map::{SqlApplicationFacts, SqlFactContext};

pub(crate) fn configure_profiles(
    root: &std::path::Path,
    config: &crate::config::v2::NoMistakesConfig,
    plan: &mut crate::codebase::check_facts::CheckFactPlan,
) -> anyhow::Result<()> {
    super::assign_prepared_embedded_sql_options(config, plan)?;
    super::configure_executor_warnings(root, config, plan)?;
    plan.postgres_schema_catalog_paths =
        super::configured_schema_catalog_paths(config, super::SCHEMA_CATALOG_RULE_IDS)?;
    Ok(())
}
