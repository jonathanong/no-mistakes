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
fn final_advisory_collection_does_not_rescan_nextjs_configs() {
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
    assert!(ordinary.is_empty(), "{ordinary:?}");
    assert_eq!(deferred, ordinary);
}

#[test]
fn advisory_option_errors_survive_both_suppression_modes() {
    let root = Path::new("/repo");
    let sources = SourceStore::new(Arc::new(FileInventory::from_paths(&[])));
    let mut config = NoMistakesConfig::default();
    config.rules.push(RuleDef {
        rule: no_mistakes::codebase::rules::AGENTS_MD_MAX_SIZE.into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{ maxLines: invalid }").unwrap(),
        ..Default::default()
    });
    for include_suppressed in [false, true] {
        assert!(
            collect(false, include_suppressed, root, &config, &[], &sources)
                .unwrap()
                .is_empty()
        );
        let error = collect(true, include_suppressed, root, &config, &[], &sources).unwrap_err();
        let message = format!("{error:#}");
        assert!(message.contains("agents-md-max-size"), "{message}");
        assert!(message.contains("options.maxLines"), "{message}");
    }
}
