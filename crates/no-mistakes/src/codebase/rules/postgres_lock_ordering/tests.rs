mod behavior;
mod false_positives;
mod join_pinning;
mod multi_target;
mod options;
mod patch_coverage;
mod pinned_prefix;
mod unanalyzable;
mod variants;
use super::directive::{
    call_offset, comment_contains_directive, contains_for_update, floor_char_boundary,
    has_safe_directive, line_start_offset, DEFAULT_SAFE_DIRECTIVE,
};
use super::scan::{
    LOCK_ORDERING_TARGET, UNANALYZABLE_TARGET, UNPARSEABLE_TARGET, UNRESOLVED_RELATION_TARGET,
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
    // New catalog fixtures live under the canonical repository fixture root;
    // retain legacy locations for pre-existing lock-ordering scenarios.
    if matches!(
        scenario,
        "fail-catalog-join"
            | "fail-catalog-order"
            | "fail-catalog-qualified-other"
            | "fail-catalog-schema-qualified"
            | "fail-pinned-key-prefix"
            | "pass-catalog"
            | "pass-catalog-of-alias"
            | "pass-catalog-unqualified"
            | "pass-pinned-key-prefix"
    ) {
        return PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/postgres/lock-ordering")
            .join(scenario);
    }
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

fn findings_for(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let file = ts_file(&root);
    check_with_files(&root, &default_config(), &[file]).unwrap()
}

fn findings_with_catalog(scenario: &str) -> Vec<RuleFinding> {
    let root = fixture(scenario);
    let files = vec![ts_file(&root), root.join("schema.json")];
    check_with_files(
        &root,
        &config_with_options("schemaCatalogPath: schema.json"),
        &files,
    )
    .unwrap()
}
