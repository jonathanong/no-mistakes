use super::collect;
use no_mistakes::codebase::ts_source::{FileInventory, SourceStore};
use no_mistakes::config::v2::schema::{RuleDef, RuleScope};
use no_mistakes::config::v2::NoMistakesConfig;
use std::path::Path;
use std::sync::Arc;

fn redirect_config(options: &str) -> NoMistakesConfig {
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: no_mistakes::codebase::rules::NEXTJS_REDIRECT_DESTINATIONS.to_string(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str(options).unwrap(),
        ..Default::default()
    });
    config
}

#[test]
fn nextjs_advisory_collection_covers_suppressed_and_ordinary_paths() {
    let root = no_mistakes::codebase::ts_resolver::normalize_path(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/rules/nextjs-redirect-destinations/fixture/dest-non-string"),
    );
    let files = no_mistakes::codebase::ts_source::walk_files(&root, &[]);
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&files)));
    let mut config = redirect_config("{ effort: low, unanalyzable: advise }");
    no_mistakes::codebase::rules::analysis_policy::prepare_rule_analysis(
        &root,
        &mut config,
        &files,
        &sources,
    )
    .unwrap();
    let ordinary = collect(true, false, &root, &config, &files, &sources).unwrap();
    let deferred = collect(true, true, &root, &config, &files, &sources).unwrap();
    assert_eq!(ordinary.len(), 1, "{ordinary:?}");
    assert_eq!(deferred, ordinary);
}
