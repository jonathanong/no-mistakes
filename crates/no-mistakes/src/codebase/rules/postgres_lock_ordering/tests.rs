mod options;
mod patch_coverage;
mod pinned_prefix;
mod unanalyzable;
mod variants;
use super::directive::{
    call_offset, comment_contains_directive, contains_for_update, floor_char_boundary,
    has_safe_directive, line_start_offset, DEFAULT_SAFE_DIRECTIVE,
};
use super::*;
use crate::config::v2::{
    schema::{RuleDef, RuleScope},
    NoMistakesConfig,
};
use std::path::{Path, PathBuf};

fn findings_for_call(
    file: &str,
    source: &str,
    call: &crate::codebase::postgres::EmbeddedSqlCall,
    opts: &CompiledOptions,
    catalog: Option<&crate::codebase::postgres::SchemaCatalog>,
) -> Vec<RuleFinding> {
    super::scan::findings_for_prepared_call(file, source, call, opts, catalog, None)
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-lock-ordering")
}

fn fixture(scenario: &str) -> PathBuf {
    fixture_root().join("fixture").join(scenario)
}

fn config_with_options(yaml: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: RULE_ID.to_string(),
        scope: Some(RuleScope::Repository),
        options: crate::codebase::postgres::tests::fixture_rule_options(yaml),
        ..Default::default()
    });
    config
}

fn default_config() -> NoMistakesConfig {
    config_with_options("{}")
}

fn ts_file(root: &Path) -> PathBuf {
    root.join("src/lock.ts")
}
