mod analysis_policy;
pub(crate) use analysis_policy::validate_low_alias;
pub use analysis_policy::{AnalysisFallback, EffectiveRuleAnalysis, Effort, Unanalyzable};
pub mod discover;
pub mod frontend_apps;
mod postgres_project_scope;
pub mod schema;
pub mod test_plan;
pub mod view;

pub(crate) use discover::{
    effective_v2_config_path_from_visible, load_v2_config_from_selected_source_store,
    load_v2_config_from_source_store, load_v2_config_with_path,
    load_v2_config_with_path_from_visible,
};
pub use discover::{find_config_root, load_v2_config, load_v2_config_from_visible};
pub use frontend_apps::{
    frontend_apps, frontend_apps_lenient, frontend_apps_or_default, FrontendApp,
};
pub use schema::NoMistakesConfig;
pub use view::ConfigView;

#[cfg(test)]
mod tests;
