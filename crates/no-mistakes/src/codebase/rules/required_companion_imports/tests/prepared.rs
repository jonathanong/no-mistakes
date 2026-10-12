use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};
use crate::codebase::dependencies::extract::{is_tsx_file, ImportExtractor};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::{FileInventory, SourceStore};
use std::sync::Arc;

fn files(root: &Path) -> Vec<PathBuf> {
    [
        "src/A.tsx",
        "src/B.tsx",
        "src/C.tsx",
        "src/D.tsx",
        "stories/value.stories.mts",
        "stories/types.stories.tsx",
        "stories/commented.stories.ts",
        "stories/fatal.stories.mts",
        "stories/legacy.stories.js",
        "stories/missing.stories.mts",
    ]
    .into_iter()
    .map(|name| root.join(name))
    .collect()
}

#[test]
fn prepared_companions_select_and_project_each_shared_input_once() {
    let root = fixture_root("prepared-shared");
    let files = files(&root);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(Arc::clone(&observer)),
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let mut plan = CheckFactPlan::default();
    configure_fact_plan(&root, &config, &files, &sources, &mut plan).unwrap();
    let cloned = plan.clone();
    assert!(Arc::ptr_eq(
        &plan.companion_import_plans,
        &cloned.companion_import_plans
    ));
    plan.include(cloned);
    assert_eq!(plan.companion_import_plans.len(), 1);
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (Vec::new(), Vec::new()),
        plan,
        None,
        Arc::clone(&sources),
    );
    let before = observer.snapshot().work;
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    let after = observer.snapshot().work;
    assert_eq!(before.get("parse.files"), after.get("parse.files"));
    assert_eq!(before.get("source.reads"), after.get("source.reads"));
    assert_eq!(after.get("companion_imports.candidate_universes"), Some(&1));
    assert_eq!(after.get("companion_imports.selections"), Some(&1));
    assert_eq!(after.get("companion_imports.literal_projections"), Some(&6));
    assert_eq!(after.get("companion_imports.literal_unions"), Some(&1));
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["src/C.tsx", "src/D.tsx"]
    );
    let parses = session.work_snapshot().parse_attempts;
    assert!(files
        .iter()
        .filter(|file| file.starts_with(root.join("src")))
        .all(|file| !parses.contains_key(file)));
    assert_eq!(
        parses.get(&root.join("stories/types.stories.tsx")),
        Some(&1)
    );
    assert_eq!(
        parses.get(&root.join("stories/value.stories.mts")),
        Some(&2)
    );
    assert_eq!(
        parses.get(&root.join("stories/legacy.stories.js")),
        Some(&2)
    );
    assert!(facts
        .get_production_imports(&root.join("stories/missing.stories.mts"))
        .unwrap()
        .is_empty());
    for file in files
        .iter()
        .filter(|file| file.starts_with(root.join("stories")) && file.exists())
    {
        let source = sources.read_path(file).unwrap();
        let extractor = if is_tsx_file(file) {
            ImportExtractor::for_tsx()
        } else {
            ImportExtractor::for_typescript()
        }
        .unwrap();
        assert_eq!(
            facts.get_production_imports(file).unwrap(),
            extractor.extract(&source).unwrap()
        );
    }
    assert_eq!(
        serde_json::to_vec(&findings).unwrap(),
        serde_json::to_vec(&check_with_files(&root, &config, &files).unwrap()).unwrap()
    );
}

#[test]
fn prepared_companion_plans_survive_union_and_fact_views() {
    let root = fixture_root("prepared-shared");
    let files = files(&root);
    let first_config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut second_config = first_config.clone();
    // The applications themselves stay identical, but effective path scope differs.
    second_config.filesystem.skip_directories = vec!["stories".to_string()];
    assert_eq!(first_config.rules, second_config.rules);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut first = CheckFactPlan::default();
    let mut second = CheckFactPlan::default();
    configure_fact_plan(&root, &first_config, &files, &sources, &mut first).unwrap();
    configure_fact_plan(&root, &second_config, &files, &sources, &mut second).unwrap();
    let first_plans = Arc::clone(&first.companion_import_plans);
    first.include(second.clone());
    assert_eq!(first.companion_import_plans.len(), 2);
    assert_eq!(first_plans.len(), 1);
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let first_facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (Vec::new(), Vec::new()),
        first,
        None,
        Arc::clone(&sources),
    );
    let second_facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (Vec::new(), Vec::new()),
        second,
        None,
        Arc::clone(&sources),
    );
    let viewed = first_facts.fact_view_with_supplemental(&second_facts);
    assert_eq!(viewed.companion_import_plans().len(), 2);
    assert_eq!(
        check_with_files_sources_and_facts(&root, &first_config, &files, &sources, Some(&viewed))
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        check_with_files_sources_and_facts(&root, &second_config, &files, &sources, Some(&viewed))
            .unwrap()
            .len(),
        4
    );
}

#[test]
fn prepared_companions_reject_missing_selection_or_sparse_facts_without_reparsing() {
    let root = fixture_root("prepared-shared");
    let files = files(&root);
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let empty = crate::codebase::check_facts::CheckFactMap::default();
    assert!(
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&empty))
            .unwrap_err()
            .to_string()
            .contains("prepared companion selection")
    );
    let mut plan = CheckFactPlan::default();
    configure_fact_plan(&root, &config, &files, &sources, &mut plan).unwrap();
    let sparse = crate::codebase::check_facts::CheckFactMap {
        companion_import_plans: plan.companion_import_plans,
        ..Default::default()
    };
    assert!(
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&sparse))
            .unwrap_err()
            .to_string()
            .contains("missing prepared companion import facts")
    );
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn prepared_companions_reuse_existing_legacy_demand_across_rule_applications() {
    let root = fixture_root("prepared-shared");
    let files = files(&root);
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let mut additional = config.rules[0].clone();
    additional.name = Some("same companion selection".to_string());
    additional.options["companionGlobs"] =
        serde_yaml::from_str("['stories/*.stories.*', 'stories/*.stories.*']").unwrap();
    config.rules.push(additional);
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let sources = Arc::new(SourceStore::new_observed(
        Arc::new(FileInventory::from_paths(&files)),
        Some(Arc::clone(&observer)),
    ));
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    // Model pre-existing production/legacy demand for the same companion files.
    let mut plan = CheckFactPlan {
        production_import_paths: Arc::new(
            files
                .iter()
                .filter(|file| file.starts_with(root.join("stories")))
                .cloned()
                .collect(),
        ),
        ..Default::default()
    };
    let existing_import_paths = Arc::clone(&plan.production_import_paths);
    configure_fact_plan(&root, &config, &files, &sources, &mut plan).unwrap();
    assert!(Arc::ptr_eq(
        &existing_import_paths,
        &plan.production_import_paths
    ));
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (Vec::new(), Vec::new()),
        plan,
        None,
        Arc::clone(&sources),
    );
    let findings =
        check_with_files_sources_and_facts(&root, &config, &files, &sources, Some(&facts)).unwrap();
    // Named applications still report the same rule ID, so identical findings
    // retain the historical sort_findings deduplication.
    assert_eq!(
        findings
            .iter()
            .map(|finding| finding.file.as_str())
            .collect::<Vec<_>>(),
        ["src/C.tsx", "src/D.tsx"]
    );
    assert_eq!(
        serde_json::to_vec(&findings).unwrap(),
        serde_json::to_vec(&check_with_files(&root, &config, &files).unwrap()).unwrap()
    );
    let work = observer.snapshot().work;
    assert_eq!(work.get("companion_imports.candidate_universes"), Some(&1));
    assert_eq!(work.get("companion_imports.selections"), Some(&1));
    assert_eq!(work.get("companion_imports.literal_projections"), Some(&6));
    assert_eq!(work.get("companion_imports.literal_unions"), Some(&1));
    assert_eq!(
        work.get("production_dependencies.legacy_import_projections"),
        Some(&3)
    );
    assert_eq!(sources.physical_read_count(), 6);
    for file in ["stories/value.stories.mts", "stories/legacy.stories.js"] {
        assert_eq!(
            session.work_snapshot().parse_attempts.get(&root.join(file)),
            Some(&2)
        );
    }
}

#[test]
fn prepared_companion_demand_is_not_activated_by_ast_only_rules() {
    let saved = fixture_root("prepared-shared");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().canonicalize().unwrap();
    crate::ast::begin_parse_count(&root);
    let findings = crate::codebase::rules::run_check(
        &root,
        Some(Path::new(".invalid-glob-no-mistakes.yml")),
        None,
    )
    .unwrap();
    let counts = crate::ast::finish_parse_count(&root);
    assert!(findings.is_empty());
    assert!(
        counts.is_empty(),
        "AST-only raw-source demand must not parse companion imports: {counts:#?}"
    );
}

#[test]
fn prepared_companions_use_discovered_external_project_companions() {
    let saved = fixture_root("prepared-external");
    let fixture = crate::test_support::materialize_saved_fixture(&saved);
    let root = fixture.path().join("root").canonicalize().unwrap();
    let external = fixture.path().join("outside").canonicalize().unwrap();
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    // Absolute external roots are supported; retain the saved relative root's
    // existing lexical behavior rather than changing resolution semantics.
    config.projects.get_mut("outside").unwrap().root =
        Some(external.to_string_lossy().into_owned());
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(&root);
    let companion = external.join("stories/A.stories.mts");
    let mut unopened = config.clone();
    unopened.rules[0].include = vec!["src/*.tsx".to_string()];
    let unopened_views = crate::check_discovery::discover_check_file_views_from_snapshot(
        &root,
        &unopened,
        &[],
        false,
        &snapshot,
    );
    assert!(!unopened_views.filesystem.contains(&companion));
    let views = crate::check_discovery::discover_check_file_views_from_snapshot(
        &root,
        &config,
        &[],
        false,
        &snapshot,
    );
    assert!(views.filesystem.contains(&companion));
    assert!(!snapshot.paths_for(&root).contains(&companion));
    let sources = snapshot.source_store_for(&root);
    let mut plan = CheckFactPlan::default();
    configure_fact_plan(&root, &config, &views.filesystem, &sources, &mut plan).unwrap();
    assert!(plan.production_import_paths.contains(&companion));
    let session = crate::codebase::analysis_session::AnalysisSession::disabled();
    let facts = collect_check_facts_with_graph_files_playwright_sources_and_session(
        &session,
        &root,
        (Vec::new(), Vec::new()),
        plan,
        None,
        Arc::clone(&sources),
    );
    assert!(check_with_files_sources_and_facts(
        &root,
        &config,
        &views.filesystem,
        &sources,
        Some(&facts)
    )
    .unwrap()
    .is_empty());
    assert!(check_with_files(&root, &config, &views.filesystem)
        .unwrap()
        .is_empty());
}
