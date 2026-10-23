use super::super::*;
use no_mistakes::codebase::ts_source::VisiblePathSnapshot;
use no_mistakes::config::v2::schema::{RuleDef, RuleScope};
use std::path::PathBuf;

#[test]
fn redirect_task_keeps_standalone_preparation_suppression_and_disabled_paths() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/nextjs-redirect-destinations/fixture/low-path-alias")
        .canonicalize()
        .unwrap();
    let files = no_mistakes::codebase::ts_source::walk_files(&root, &[]);
    let snapshot = VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let mut configured = NoMistakesConfig::default();
    configured.rules.push(RuleDef {
        rule: rules::NEXTJS_REDIRECT_DESTINATIONS.into(),
        scope: Some(RuleScope::Repository),
        options: serde_yaml::from_str("{ effort: low, unanalyzable: advise }").unwrap(),
        ..Default::default()
    });
    rules::analysis_policy::prepare_rule_analysis(&root, &mut configured, &files, &sources)
        .unwrap();
    for enabled in [false, true] {
        for has_rule in [false, true] {
            for deferred in [false, true] {
                let empty = NoMistakesConfig::default();
                let config = if has_rule { &configured } else { &empty };
                let result = run_filesystem_rules_check_with_facts(
                    &root,
                    config,
                    enabled,
                    &files,
                    rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
                        snapshot: &snapshot,
                        next_config_resolver: None,
                        vitest_catalog: None,
                        sources: std::sync::Arc::clone(&sources),
                        workflow_documents: None,
                        tsconfig_gate_project_inputs: None,
                        config_path: None,
                    },
                    None,
                    deferred,
                )
                .unwrap();
                let expected = if enabled && has_rule { 2 } else { 0 };
                assert_eq!(result.findings.len(), expected);
                assert_eq!(result.advisories.len(), expected);
                assert!(result.warning.is_none());
            }
        }
    }
}

#[test]
fn redirect_task_propagates_rule_and_other_filesystem_errors() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/nextjs-redirect-destinations/fixture/low-path-alias")
        .canonicalize()
        .unwrap();
    let files = no_mistakes::codebase::ts_source::walk_files(&root, &[]);
    let snapshot = VisiblePathSnapshot::from_paths(&root, &files);
    let sources = snapshot.source_store_for(&root);
    let catalog = no_mistakes::codebase::ts_resolver::TsConfigCatalog::from_visible_and_sources(
        &root,
        std::slice::from_ref(&root),
        &files,
        &sources,
    );
    let session = no_mistakes::codebase::analysis_session::AnalysisSession::disabled();
    let resolver = rules::nextjs_redirect_destinations::PreparedConfigResolver::new(
        &catalog, &files, &session,
    );
    for prepared_resolver in [None, Some(&resolver)] {
        for deferred in [false, true] {
            for (rule, options) in [
                (rules::NEXTJS_REDIRECT_DESTINATIONS, "{ configPath: false }"),
                (rules::RUST_MAX_LINES_PER_FILE, "{ srcMax: wrong-type }"),
            ] {
                let mut config = NoMistakesConfig::default();
                config.rules.push(RuleDef {
                    rule: rule.into(),
                    scope: Some(RuleScope::Repository),
                    options: serde_yaml::from_str(options).unwrap(),
                    ..Default::default()
                });
                let result = run_filesystem_rules_check_with_facts(
                    &root,
                    &config,
                    true,
                    &files,
                    rules::filesystem_dispatch::PreparedFilesystemRuleInputs {
                        snapshot: &snapshot,
                        next_config_resolver: prepared_resolver,
                        vitest_catalog: None,
                        sources: std::sync::Arc::clone(&sources),
                        workflow_documents: None,
                        tsconfig_gate_project_inputs: None,
                        config_path: None,
                    },
                    None,
                    deferred,
                );
                let error = result
                    .err()
                    .expect("invalid options must propagate through the task");
                assert!(error.to_string().contains(rule), "{error:#}");
            }
        }
    }
}
