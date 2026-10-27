use super::*;
use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_playwright_and_sources,
    collect_check_facts_with_graph_files_playwright_sources_and_session, CheckFactPlan,
};

fn fixture() -> PathBuf {
    crate::test_support::rule_fixture_root(RULE_ID)
}
fn configured(name: &str) -> NoMistakesConfig {
    crate::config::v2::load_v2_config(&fixture(), Some(&fixture().join(name))).unwrap()
}
fn files() -> Vec<PathBuf> {
    [
        "src/directives.tsx",
        "src/suppressed.ts",
        "src/file-suppressed.ts",
        "src/recovered.ts",
        "src/multiline.ts",
        "src/line-endings.ts",
        "src/quoted.ts",
        "src/html.cjs",
        "src/ts-prefix.ts",
        "src/ts-blocks.ts",
        "outside/excluded.ts",
    ]
    .iter()
    .map(|p| fixture().join(p))
    .collect()
}

#[test]
fn invalid_options_fail_before_reading_sources() {
    for config in [
        "missing-files.yml",
        "empty-files.yml",
        "bad-glob.yml",
        "bad-directive.yml",
        "empty-id.yml",
    ] {
        assert!(
            required_files(&fixture(), &configured(config), &files()).is_err(),
            "{config}"
        );
    }
}
#[test]
fn prepared_comments_are_borrowed_without_second_reads_or_parses() {
    let files = files();
    let config = configured("default.yml");
    let sources = super::super::source_store_for_files(&files);
    let paths = required_files(&fixture(), &config, &files).unwrap();
    let observer = crate::diagnostics::InvocationObserver::new(true);
    let session =
        crate::codebase::analysis_session::AnalysisSession::new(Some(Arc::clone(&observer)));
    let facts = {
        let _guard = crate::diagnostics::InvocationGuard::install(Arc::clone(&observer));
        collect_check_facts_with_graph_files_playwright_sources_and_session(
            &session,
            &fixture(),
            (paths.iter().cloned().collect(), Vec::new()),
            CheckFactPlan {
                lint_suppression_paths: Arc::clone(&paths),
                graph: crate::codebase::ts_source::facts::TsFactPlan::imports(),
                ..Default::default()
            },
            None,
            Arc::clone(&sources),
        )
    };
    let before = observer.snapshot().work;
    assert_eq!(before.get("parse.files"), Some(&(paths.len() as u64)));
    let reads = sources.physical_read_count();
    let actual =
        check_prepared(&fixture(), &config, &files, &sources, Some(&facts), false).unwrap();
    assert!(!actual.is_empty());
    assert_eq!(sources.physical_read_count(), reads);
    assert_eq!(
        observer.snapshot().work.get("parse.files"),
        before.get("parse.files")
    );
}

#[test]
fn missing_prepared_facts_never_trigger_private_reads_or_parses() {
    let files = files();
    let config = configured("default.yml");
    let sources = super::super::source_store_for_files(&files);
    let empty = collect_check_facts_with_graph_files_playwright_and_sources(
        &fixture(),
        Vec::new(),
        Vec::new(),
        CheckFactPlan::default(),
        None,
        Arc::clone(&sources),
    );
    for facts in [None, Some(&empty)] {
        let error =
            check_prepared(&fixture(), &config, &files, &sources, facts, false).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("missing prepared comment facts for src/"),
            "{error}"
        );
    }
    assert_eq!(sources.physical_read_count(), 0);
}

#[test]
fn lexical_and_normalized_source_aliases_share_the_prepared_identity() {
    let mut files = files();
    files.extend(
        files
            .clone()
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path)),
    );
    let actual = check_with_files(&fixture(), &configured("quoted.yml"), &files).unwrap();
    assert_eq!(actual.len(), 7, "{actual:#?}");
    assert!(actual.iter().all(|finding| finding.file == "src/quoted.ts"));
}
