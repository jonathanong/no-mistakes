use super::*;
use crate::config::v2::schema::{RuleDef, RuleScope};

fn config_with(rule: &str) -> crate::config::v2::NoMistakesConfig {
    let mut config = crate::config::v2::NoMistakesConfig::default();
    config.filesystem.skip_directories = vec!["generated".to_string()];
    config.rules.push(RuleDef {
        rule: rule.to_string(),
        scope: Some(RuleScope::Repository),
        options: if rule == super::super::REQUIRED_ENTRYPOINT_REACHABILITY {
            serde_yaml::from_str("sourceGlobs: [generated/worker.ts]\nentrypoints: [src/api.ts]\n")
                .unwrap()
        } else {
            Default::default()
        },
        ..Default::default()
    });
    config
}

fn graph_files_for(config: &crate::config::v2::NoMistakesConfig) -> Vec<std::path::PathBuf> {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("generated")).unwrap();
    std::fs::write(root.join("src/api.ts"), "export {};\n").unwrap();
    std::fs::write(root.join("generated/worker.ts"), "export {};\n").unwrap();

    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let scoped = crate::codebase::ts_source::discover_files_from_visible(
        root,
        &config.filesystem.skip_directories,
        &visible_paths,
    );
    standalone_graph_files(
        root,
        config,
        canonical_graph_plan(config),
        &visible_paths,
        &scoped,
    )
}

#[test]
fn dynamic_only_graph_files_honor_skipped_directories() {
    let files = graph_files_for(&config_with(TEST_NO_UNMOCKED_DYNAMIC_IMPORTS));
    assert_eq!(files.len(), 1);
    assert!(files[0].ends_with("src/api.ts"));
}

#[test]
fn full_universe_graph_files_include_skipped_directories() {
    let files = graph_files_for(&config_with(super::super::REQUIRED_ENTRYPOINT_REACHABILITY));
    assert_eq!(files.len(), 2);
    assert!(files
        .iter()
        .any(|file| file.ends_with("generated/worker.ts")));
}
