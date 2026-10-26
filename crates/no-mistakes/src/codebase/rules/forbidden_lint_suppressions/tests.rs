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
fn run(name: &str) -> Vec<RuleFinding> {
    check_with_files(&fixture(), &configured(name), &files()).unwrap()
}
#[test]
fn actual_comment_headers_report_every_protected_id_and_form() {
    let findings = run("default.yml");
    let actual: Vec<_> = findings
        .iter()
        .filter(|f| f.file == "src/directives.tsx")
        .map(|f| (f.line, f.target.as_deref().unwrap()))
        .collect();
    assert_eq!(
        actual,
        [
            (1, "react/purity"),
            (2, "react/purity"),
            (3, "react-hooks-js/refs"),
            (3, "react/purity"),
            (4, "react/purity"),
            (5, "react-hooks-js/purity"),
            (6, "react/purity"),
            (20, "react-hooks-js/refs")
        ]
    );
    assert!(!findings
        .iter()
        .any(|f| f.file.contains("suppressed") || f.file.contains("outside")));
    assert!(findings
        .iter()
        .any(|f| f.file == "src/recovered.ts" && f.line == 1));
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.file == "src/multiline.ts")
            .map(|f| f.line)
            .collect::<Vec<_>>(),
        [2, 2]
    );
    assert_eq!(
        findings
            .iter()
            .filter(|f| f.file == "src/line-endings.ts")
            .map(|f| f.line)
            .collect::<Vec<_>>(),
        [1, 3, 5]
    );
}
#[test]
fn blanket_and_typescript_are_explicit_opt_ins() {
    let blanket = run("blanket.yml");
    // Non-comma lists are malformed rule text, never blanket disables.
    assert!(!blanket
        .iter()
        .any(|f| f.file == "src/quoted.ts" && [3, 7].contains(&f.line)));
    assert_eq!(
        blanket
            .iter()
            .filter(|f| f.target.as_deref() == Some("*"))
            .map(|f| f.line)
            .collect::<Vec<_>>(),
        [7, 8]
    );
    let ts = run("ts.yml");
    assert_eq!(
        ts.iter()
            .filter(|f| f.file == "src/directives.tsx")
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [(9, "@ts-ignore"), (11, "@ts-expect-error")]
    );
    for config in ["none.yml", "no-matches.yml", "excluded.yml"] {
        assert!(run(config).is_empty(), "{config}");
    }
    assert!(run("custom.yml")
        .iter()
        .all(|f| f.message.ends_with("Keep compiler checks enabled.")));
}

#[test]
fn explicit_lint_directive_selection_preserves_default_forms() {
    assert_eq!(run("explicit.yml"), run("default.yml"));
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
fn quoted_ids_and_embedded_double_hyphens_preserve_rule_identity() {
    let actual = run("quoted.yml");
    assert_eq!(
        actual
            .iter()
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [
            (1, "react-hooks-js/refs"),
            (1, "react/purity"),
            (2, "react-hooks-js/refs"),
            (2, "react/purity"),
            (4, "custom/no--bad"),
            (5, "react/purity"),
            (6, "react/purity")
        ]
    );
}
#[test]
fn typescript_uses_final_block_line_and_triple_slash_comments() {
    let actual = run("ts.yml");
    assert_eq!(
        actual
            .iter()
            .filter(|f| f.file == "src/ts-blocks.ts")
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [
            (1, "@ts-ignore"),
            (4, "@ts-ignore"),
            (7, "@ts-expect-error"),
            (13, "@ts-expect-error")
        ]
    );
}

#[test]
fn annex_b_comments_remain_ast_backed_lint_directives() {
    let actual = run("html.yml");
    assert_eq!(
        actual
            .iter()
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [(1, "react/purity"), (3, "react/purity")]
    );
}
#[test]
fn typescript_accepts_compiler_prefix_markers_without_lint_boundaries() {
    let actual = run("ts.yml");
    assert_eq!(
        actual
            .iter()
            .filter(|f| f.file == "src/ts-prefix.ts")
            .map(|f| (f.line, f.target.as_deref().unwrap()))
            .collect::<Vec<_>>(),
        [(1, "@ts-ignore")]
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
