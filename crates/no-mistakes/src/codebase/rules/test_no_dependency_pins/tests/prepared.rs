use super::*;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

fn saved() -> tempfile::TempDir {
    crate::test_support::materialize_saved_fixture(&fixture_root().join("prepared-jsx"))
}

#[test]
fn prepared_jsx_ranges_survive_eviction_and_preserve_recovery_and_read_failures() {
    let fixture = saved();
    let root = fixture.path().canonicalize().unwrap();
    let config =
        crate::config::v2::load_v2_config(&root, Some(Path::new(".no-mistakes.yml"))).unwrap();
    let mut files = crate::codebase::ts_source::discover_files(&root, &[]);
    files.push(root.join("src/missing.test.tsx"));
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(observer.clone()),
    ));
    let session = AnalysisSession::new(Some(observer.clone()));
    let mut plan = CheckFactPlan::default();
    configure_fact_plan(&root, &config, &files, &mut plan).unwrap();
    assert_eq!(plan.jsx_text_paths.len(), 5);
    assert!(!plan.jsx_text_paths.contains(&root.join("src/helper.tsx")));
    assert!(!plan
        .jsx_text_paths
        .contains(&root.join("src/excluded.test.tsx")));
    let selected = plan.jsx_text_paths.iter().cloned().collect();
    crate::ast::begin_parse_count(&root);
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (selected, vec![]),
        plan,
        None,
        sources.clone(),
    );
    let prepared =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
    let recovered = &facts.ts[&root.join("src/recovered.test.tsx")].ts;
    assert!(recovered.parse_error.is_some());
    assert!(!recovered.jsx_text_ranges.as_ref().unwrap().is_empty());
    assert!(facts.ts[&root.join("src/missing.test.tsx")]
        .ts
        .operational_error
        .is_some());
    assert!(observer
        .source_read_snapshot()
        .values()
        .all(|reads| *reads == 1));
    assert_eq!(observer.snapshot().work["parse.files"], 4);
    let standalone = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(
        serde_json::to_value(prepared).unwrap(),
        serde_json::to_value(standalone).unwrap()
    );
}

#[test]
fn custom_patterns_request_no_jsx_parsing_and_union_retains_default_demand() {
    let fixture = saved();
    let root = fixture.path().canonicalize().unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let custom =
        crate::config::v2::load_v2_config(&root, Some(Path::new(".custom-no-mistakes.yml")))
            .unwrap();
    let mut plan = CheckFactPlan::default();
    configure_fact_plan(&root, &custom, &files, &mut plan).unwrap();
    assert!(plan.jsx_text_paths.is_empty());
    crate::ast::begin_parse_count(&root);
    let findings = check_with_files(&root, &custom, &files).unwrap();
    assert!(crate::ast::finish_parse_count(&root).is_empty());
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].import.as_deref(), Some("CUSTOM_PIN"));
    let mut mixed = custom.clone();
    mixed.rules.extend(config_with_options("{}").rules);
    configure_fact_plan(&root, &mixed, &files, &mut plan).unwrap();
    assert!(plan
        .jsx_text_paths
        .contains(&root.join("src/pins.test.tsx")));
    assert!(!plan.jsx_text_paths.contains(&root.join("src/helper.tsx")));
}

#[test]
fn prepared_jsx_invalid_configuration_preserves_filter_then_pattern_errors() {
    let fixture = saved();
    let root = fixture.path().canonicalize().unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    for config_path in [
        ".invalid-filter-no-mistakes.yml",
        ".invalid-pattern-no-mistakes.yml",
    ] {
        // Public loading rejects invalid rule globs first; deserialize the saved
        // config directly here to compare the lower-layer precedence as well.
        let config: NoMistakesConfig =
            serde_yaml::from_str(&std::fs::read_to_string(root.join(config_path)).unwrap())
                .unwrap();
        let mut plan = CheckFactPlan::default();
        let prepared = configure_fact_plan(&root, &config, &files, &mut plan).unwrap_err();
        let standalone = check_with_files(&root, &config, &files).unwrap_err();
        assert_eq!(format!("{prepared:#}"), format!("{standalone:#}"));
        assert!(prepared.to_string().contains("include"), "{prepared:#}");
    }
}

#[test]
fn standalone_filesystem_dispatch_prepares_jsx_ranges_once() {
    let fixture = saved();
    let root = fixture.path().canonicalize().unwrap();
    let config =
        crate::config::v2::load_v2_config(&root, Some(Path::new(".no-mistakes.yml"))).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    crate::ast::begin_parse_count(&root);
    let findings = crate::codebase::rules::filesystem_dispatch::run_filesystem_rules_with_config(
        &root, &config, &files,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert_eq!(findings.len(), 4, "{findings:#?}");
    assert_eq!(counts.len(), 4, "{counts:#?}");
    assert!(counts.values().all(|count| *count == 1), "{counts:#?}");
}

#[test]
fn prepared_jsx_mixed_legacy_symbols_and_production_imports_preserve_standard_masking() {
    assert_mixed_masking(true);
}

#[test]
fn prepared_jsx_legacy_symbols_without_production_imports_preserve_standard_masking() {
    assert_mixed_masking(false);
}

fn assert_mixed_masking(production_imports: bool) {
    let fixture = saved();
    let root = fixture.path().canonicalize().unwrap();
    let config = config_with_options("include: ['src/*.tsx', 'src/*.jsx']");
    let files = [
        root.join("src/pins.test.tsx"),
        root.join("src/pins.test.jsx"),
        root.join("src/recovered.test.tsx"),
        root.join("src/types.jsx"),
    ];
    let session = AnalysisSession::disabled();
    let sources = super::super::super::source_store_for_files(&files);
    let mut plan = CheckFactPlan {
        symbols: true,
        legacy_symbol_paths: files.iter().cloned().collect(),
        production_import_paths: if production_imports {
            Arc::new(files.iter().cloned().collect())
        } else {
            Default::default()
        },
        ..Default::default()
    };
    configure_fact_plan(&root, &config, &files, &mut plan).unwrap();
    crate::ast::begin_parse_count(&root);
    let facts = crate::ast::with_request_parse_cache(|| {
        collect_check_facts_with_graph_files_playwright_sources_and_session(
            &session,
            &root,
            (files.to_vec(), vec![]),
            plan,
            None,
            sources.clone(),
        )
    });
    let prepared =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    // TSX shares the equivalent historical mode; JSX retains one standard parse and one legacy projection.
    assert_eq!(counts.get(&files[0]), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&files[1]), Some(&2), "{counts:#?}");
    assert_eq!(counts.get(&files[2]), Some(&1), "{counts:#?}");
    assert_eq!(counts.get(&files[3]), Some(&2), "{counts:#?}");
    assert!(facts.ts[&files[3]].ts.parse_error.is_some());
    assert!(facts.ts[&files[3]]
        .ts
        .jsx_text_ranges
        .as_ref()
        .unwrap()
        .is_empty());
    assert!(facts.ts[&files[3]].legacy_symbols.is_some());
    for path in &files[..3] {
        assert!(facts.ts[path]
            .ts
            .jsx_text_ranges
            .as_ref()
            .is_some_and(|ranges| !ranges.is_empty()));
    }
    assert!(facts.ts[&files[0]].symbols.is_some());
    assert_eq!(
        facts.ts[&files[1]].production_imports.is_some(),
        production_imports
    );
    let standalone = check_with_files_and_sources(&root, &config, &files, &sources).unwrap();
    assert_eq!(
        serde_json::to_value(prepared).unwrap(),
        serde_json::to_value(standalone).unwrap()
    );
}
